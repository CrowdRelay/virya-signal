#!/usr/bin/env python3
"""Keep the Web/WASM and native Tauri Fan Home DTOs field-for-field aligned."""

from __future__ import annotations

import re
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
WEB_MODELS = ROOT / "src" / "models.rs"
NATIVE_MODELS = ROOT / "src-tauri" / "src" / "models" / "session_fan.rs"

STRUCTS = (
    "FanHomeData",
    "FanHomeProfile",
    "FanHomeEvent",
    "FanHomeSynesthesia",
    "FanHomeReferral",
    "FanHomeCounts",
)


def struct_fields(source: str, name: str) -> tuple[str, ...]:
    match = re.search(
        rf"pub\s+struct\s+{re.escape(name)}\s*\{{(?P<body>.*?)^\}}",
        source,
        re.MULTILINE | re.DOTALL,
    )
    if match is None:
        raise AssertionError(f"missing public struct {name}")
    return tuple(
        re.findall(r"^\s*pub\s+([A-Za-z_][A-Za-z0-9_]*)\s*:", match.group("body"), re.MULTILINE)
    )


class FanHomeContractParityTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.web = WEB_MODELS.read_text()
        cls.native = NATIVE_MODELS.read_text()

    def test_fan_home_dto_fields_match_native_contract(self) -> None:
        for name in STRUCTS:
            with self.subTest(struct=name):
                self.assertEqual(
                    struct_fields(self.web, name),
                    struct_fields(self.native, name),
                    f"{name} drifted between src/models.rs and native session_fan.rs",
                )

    def test_profile_locale_is_part_of_both_contracts(self) -> None:
        for source in (self.web, self.native):
            self.assertIn("locale", struct_fields(source, "FanHomeProfile"))


if __name__ == "__main__":
    unittest.main()
