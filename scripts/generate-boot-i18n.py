#!/usr/bin/env python3
"""Generate boot-critical and runtime web translation catalogs.

Native Tauri keeps compiling the canonical Rust PL/EN catalogs directly. The web
build keeps only the tiny splash/recovery vocabulary on the parser-blocking path;
the full catalog lives in a separate runtime asset declared after the Trunk Rust
entrypoint so WASM discovery is not delayed by translation payload download.
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


def frozen_object(value: dict[str, str]) -> str:
    return "Object.freeze(" + compact(value) + ")"


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
    runtime = (
        "window.__VIRYA_RUNTIME_I18N__ = Object.freeze({"
        f'"pl":{frozen_object(pl)},"en":{frozen_object(en)}'
        "});\n"
    )
    (ROOT / "runtime-i18n.js").write_text(runtime, encoding="utf-8")


if __name__ == "__main__":
    main()
