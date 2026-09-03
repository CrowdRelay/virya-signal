#!/usr/bin/env python3
"""Generate boot-critical and runtime web translation catalogs.

Native Tauri keeps compiling the canonical Rust PL/EN catalogs directly. The web
build keeps only the tiny splash/recovery vocabulary on the parser-blocking path.
The runtime catalog stores its keys once and fetches only the selected language's
parallel value array. This keeps the two large, identical key sets out of the
initial WebView parse while preserving live language switching.
"""
from __future__ import annotations

import hashlib
import json
import re
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
]


def compact(value: object) -> str:
    return json.dumps(
        value,
        ensure_ascii=False,
        separators=(",", ":"),
        sort_keys=True,
    )


# The runtime catalogs are fetched with `cache: "force-cache"`, so this token is
# the only thing that can invalidate them in a WebView that already has them.
# It used to be a hand-edited literal, and it was still "0.4.2" while the app
# shipped 0.5.x — every string added or corrected in between was invisible to
# anyone who had already opened the app once. Deriving it from the catalog
# content means it cannot be forgotten: change a translation, get a new URL.
RUNTIME_VERSION_PREFIX = "runtime-i18n-v4"


def catalog_version(keys: list[str], pl: list[str], en: list[str]) -> str:
    digest = hashlib.sha256(compact([keys, pl, en]).encode("utf-8")).hexdigest()
    return f"{RUNTIME_VERSION_PREFIX}-{digest[:16]}"


def write_json(name: str, value: object) -> None:
    (ROOT / name).write_text(compact(value) + "\n", encoding="utf-8")


# index.html carries a ?v= token per boot script. All three were hand-edited
# literals reading 0.4.2 while the app shipped 0.5.x, so a changed boot script —
# boot-i18n.js changes with every catalog edit — kept its old URL and an
# installed app could keep serving the previous copy from cache. Same rule as
# the runtime catalog token: derive it from the file's own bytes.
SCRIPT_TAG = '<script defer src="{name}?v={token}"></script>'
BOOT_SCRIPTS = ("boot-i18n.js", "boot.js", "runtime-i18n.js")


def file_token(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()[:16]


def pin_boot_script_tokens() -> None:
    index = ROOT / "index.html"
    text = index.read_text(encoding="utf-8")
    for name in BOOT_SCRIPTS:
        stem = name.removesuffix(".js")
        token = f"{stem}-{file_token(ROOT / name)}"
        pattern = re.compile(
            r'<script defer src="' + re.escape(name) + r'\?v=[^"]*"></script>'
        )
        replacement = SCRIPT_TAG.format(name=name, token=token)
        text, count = pattern.subn(replacement, text, count=1)
        if count != 1:
            raise SystemExit(f"index.html has no versioned <script> for {name}")
    index.write_text(text, encoding="utf-8")


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
    pl_values = [pl[key] for key in keys]
    en_values = [en[key] for key in keys]
    write_json("runtime-i18n-keys.json", keys)
    write_json("runtime-i18n-pl.json", pl_values)
    write_json("runtime-i18n-en.json", en_values)
    runtime_version = catalog_version(keys, pl_values, en_values)

    runtime = f'''(() => {{
  "use strict";

  const VERSION = "{runtime_version}";
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
  // The catalog arrives after first paint, and until it does `text` answers a
  // miss with the key itself. Without this the app was never told the catalog
  // had landed, so whichever labels rendered during the load stayed as raw
  // identifiers. Same event the language switch already uses.
  void ready.then(() => dispatchReady());

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
    pin_boot_script_tokens()


if __name__ == "__main__":
    main()
