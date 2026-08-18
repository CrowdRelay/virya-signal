#!/usr/bin/env python3
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

def read(path: str) -> str:
    return (ROOT / path).read_text(encoding="utf-8")

class LatarnikVaultIsolation(unittest.TestCase):
    def test_beacon_vault_has_distinct_client_key_and_files(self):
        vault = read("src-tauri/src/vault.rs")
        for value in (
            'b"virya-signal-beacon"', 'b"beacon-profile-v1"',
            '"beacon.vault.hold"', '"beacon.vault.salt"',
        ):
            self.assertIn(value, vault)
        self.assertIn("pub fn remove_beacon", vault)
        remove = vault.split("pub fn remove_beacon", 1)[1].split("fn operator_vault_path", 1)[0]
        self.assertNotIn("fan.vault", remove)
        self.assertNotIn("operator.vault", remove)

    def test_beacon_replace_is_transactional(self):
        vault = read("src-tauri/src/vault.rs")
        replace = vault.split("pub fn replace_beacon", 1)[1].split("pub fn", 1)[0]
        self.assertIn("vault_backup", replace)
        self.assertIn("salt_backup", replace)
        self.assertIn("move_if_present", replace)
        self.assertGreaterEqual(replace.count("move_if_present(&vault_backup, &vault_path)"), 2)
        self.assertIn("move_if_present(&salt_backup, &salt_path)", replace)

    def test_launcher_exposes_three_separate_principals(self):
        model = read("src-tauri/src/models/session_fan.rs")
        misc = read("src-tauri/src/commands/misc.rs")
        for field in ("operator", "fan", "beacon"):
            self.assertIn(f"pub {field}:", model)
        self.assertIn("beacon: BeaconSessionStatus", misc)

if __name__ == "__main__":
    unittest.main()
