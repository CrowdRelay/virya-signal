#!/usr/bin/env python3
from pathlib import Path
import re
import unittest

ROOT = Path(__file__).resolve().parents[1]


class SignalUiPolishTests(unittest.TestCase):
    def test_type_floor_touch_targets_and_operator_markers_hold(self) -> None:
        css = (ROOT / "styles.css").read_text()
        operator = (ROOT / "src/app/operator.rs").read_text()
        scanner = (ROOT / "src/app/scanner.rs").read_text()
        errors = []
        if not all(
            x in css for x in ["--type-micro: 12px", "--type-meta: 13px", "--touch-min: 44px"]
        ):
            errors.append("missing type/touch tokens")
        small = re.findall(r"font-size:\s*(?:[7-9]|10|11)px\s*;", css)
        if small:
            errors.append(f"legacy sub-12px font sizes remain: {small[:5]}")
        for marker in [
            "show-mode-status-grid",
            "eligible_passes",
            "pending_scans",
            "synced_scans",
            "scan_conflicts",
        ]:
            if marker not in operator and marker not in scanner and marker not in css:
                errors.append(f"missing {marker}")
        if "min-height: 300px" not in css and "min-height: 380px" not in css:
            errors.append("scanner primary target regressed")
        if errors:
            self.fail("; ".join(errors))


if __name__ == "__main__":
    unittest.main()
