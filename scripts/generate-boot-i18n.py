#!/usr/bin/env python3
from __future__ import annotations

import json
from pathlib import Path

from i18n_catalog import load_catalog_pair

ROOT = Path(__file__).resolve().parents[1]
KEYS = [
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


def main() -> None:
    pl, en = load_catalog_pair(ROOT)
    payload = {
        "pl": {key: pl[key] for key in KEYS},
        "en": {key: en[key] for key in KEYS},
    }
    output = "window.__VIRYA_BOOT_I18N__ = Object.freeze(" + json.dumps(
        payload, ensure_ascii=False, separators=(",", ":")
    ) + ");\n"
    (ROOT / "boot-i18n.js").write_text(output, encoding="utf-8")


if __name__ == "__main__":
    main()
