#!/usr/bin/env python3
"""Generate boot-critical and runtime web translation catalogs.

Native Tauri keeps compiling the canonical Rust PL/EN catalogs directly. The web
build keeps only the tiny splash/recovery vocabulary on the parser-blocking path.
The runtime catalog stores its keys once and fetches only the selected language's
parallel value array. This keeps the two large, identical key sets out of the
initial WebView parse while preserving live language switching.
"""
from __future__ import annotations

import json
from pathlib import Path

from i18n_catalog import load_catalog_pair

ROOT = Path(__file__).resolve().parents[1]
BOOT_KEYS = [
    "boot_previous_terminated",
    "boot_phase_wasm_loading",
    "boot_phase_wasm_entered",
    "boot_phase_wasm_initialized",
    "boot_unknown_error",
    "boot_start_stopped",
    "boot_module_not_started",
    "boot_engine_load_failed",
    "boot_engine_no_interface",
    "boot_interface_incomplete",
    "boot_start_incomplete",
    "boot_stage_retry_detail",
    "boot_retry_failed",
    "boot_retry_blocked_detail",
    "boot_almost_ready",
    "boot_initial_status",
    "boot_retry_button",
    "boot_noscript",
]


def compact(value: object) -> str:
    return json.dumps(
        value,
        ensure_ascii=False,
        separators=(",", ":"),
        sort_keys=True,
    )


RUNTIME_VERSION = "0.4.2-runtime-i18n-v3"


def write_json(name: str, value: object) -> None:
    (ROOT / name).write_text(compact(value) + "\n", encoding="utf-8")


def main() -> None:
    pl, en = load_catalog_pair(ROOT)
    boot = {
        "pl": {key: pl[key] for key in BOOT_KEYS},
        "en": {key: en[key] for key in BOOT_KEYS},
    }
    (ROOT / "boot-i18n.js").write_text(
        "window.__VIRYA_BOOT_I18N__ = Object.freeze(" + compact(boot) + ");\n",
        encoding="utf-8",
    )
    # Keep the order stable, so the matching PL/EN value arrays can be safely
    # indexed without repeating every verbose message identifier twice.
    keys = sorted(pl)
    write_json("runtime-i18n-keys.json", keys)
    write_json("runtime-i18n-pl.json", [pl[key] for key in keys])
    write_json("runtime-i18n-en.json", [en[key] for key in keys])

    runtime = f'''(() => {{
  "use strict";

  const VERSION = "{RUNTIME_VERSION}";
  const LANGUAGE_STORAGE_KEY = "virya:language:v1";
  const catalogs = Object.create(null);
  const pending = Object.create(null);
  let keys = null;

  const language = (value) => value === "en" ? "en" : "pl";
  const asset = (name) => `${{name}}?v=${{VERSION}}`;
  const readJson = async (name) => {{
    const response = await fetch(asset(name), {{ cache: "force-cache", credentials: "same-origin" }});
    if (!response.ok) throw new Error(`i18n asset failed: ${{response.status}}`);
    return response.json();
  }};
  const indexOf = (key) => {{
    let low = 0;
    let high = keys.length - 1;
    while (low <= high) {{
      const middle = (low + high) >>> 1;
      const candidate = keys[middle];
      if (candidate === key) return middle;
      if (candidate < key) low = middle + 1;
      else high = middle - 1;
    }}
    return -1;
  }};
  const load = (requested) => {{
    const selected = language(requested);
    if (catalogs[selected]) return Promise.resolve(selected);
    if (pending[selected]) return pending[selected];
    const task = (async () => {{
      const [loadedKeys, values] = await Promise.all([
      keys ? Promise.resolve(keys) : readJson("runtime-i18n-keys.json"),
      readJson(`runtime-i18n-${{selected}}.json`),
      ]);
      if (!Array.isArray(loadedKeys) || !Array.isArray(values) || loadedKeys.length !== values.length
        || !loadedKeys.every((key) => typeof key === "string")
        || !values.every((value) => typeof value === "string")) {{
        throw new Error("invalid i18n catalog");
      }}
      keys = loadedKeys;
      catalogs[selected] = values;
      return selected;
    }})();
    pending[selected] = task;
    void task.finally(() => {{ delete pending[selected]; }});
    return task;
  }};
  const preferred = () => {{
    try {{ return language(window.localStorage?.getItem(LANGUAGE_STORAGE_KEY)); }} catch {{ return "pl"; }}
  }};
  const dispatchReady = () => {{
    try {{ window.dispatchEvent(new CustomEvent("virya:language-change")); }} catch {{}}
  }};
  const loadWithFallback = (requested) => load(requested).catch(() => load("pl"));
  let ready = loadWithFallback(preferred());

  window.__VIRYA_RUNTIME_I18N__ = Object.freeze({{
    ready: () => ready,
    requestLanguage(requested) {{
      const request = loadWithFallback(requested);
      ready = request;
      void request.then(() => {{ if (ready === request) dispatchReady(); }});
      return request;
    }},
    text(requested, key) {{
      if (!keys) return key;
      const values = catalogs[language(requested)] || catalogs.pl;
      const index = values ? indexOf(key) : -1;
      return index >= 0 ? values[index] : key;
    }},
  }});
}})();
'''
    (ROOT / "runtime-i18n.js").write_text(runtime, encoding="utf-8")


if __name__ == "__main__":
    main()
