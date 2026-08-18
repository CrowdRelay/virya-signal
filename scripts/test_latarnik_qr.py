#!/usr/bin/env python3
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

def read(path: str) -> str:
    return (ROOT / path).read_text(encoding="utf-8")

class LatarnikQrContracts(unittest.TestCase):
    def test_qr_exchange_is_native_owned_before_resume(self):
        ffi = read("src/bridge/ffi.rs")
        client = read("src/bridge/client.rs")
        native = read("src-tauri/src/commands/beacon.rs")
        self.assertIn("viryaScanAndConfirmBeacon", ffi)
        self.assertIn("core.invoke('beacon_confirm_scanned'", ffi)
        scan = ffi.split("export async function viryaScanAndConfirmBeacon", 1)[1].split("export ", 1)[0]
        self.assertLess(scan.index("beacon_confirm_scanned"), scan.index("virya:resume"))
        self.assertIn("pub async fn scan_and_confirm_beacon()", client)
        self.assertIn("beacon_confirm_scanned", native)
        self.assertIn("persist_exchanged_beacon", native)

    def test_invite_parser_is_host_and_path_allowlisted(self):
        native = read("src-tauri/src/commands/beacon.rs")
        parser = native.split("fn normalize_invite", 1)[1].split("fn validate_api_base", 1)[0]
        for token in ('"https"', '"virya.music"', '"www.virya.music"', '"/latarnik"', '"/pl/latarnik"'):
            self.assertIn(token, parser)
        self.assertIn("fragment().is_some()", parser)
        self.assertIn('url.username() != ""', parser)
        self.assertIn("query_pairs", parser)

    def test_android_app_link_is_verified_and_one_time(self):
        prepare = read("scripts/prepare-android.py")
        kotlin = read("src-tauri/android-push/SignalPushPlugin.kt")
        rust_plugin = read("src-tauri/src/push_plugin.rs")
        self.assertIn('autoVerify', prepare)
        self.assertIn('"virya.music"', prepare)
        self.assertNotIn('"www.virya.music"', prepare)
        self.assertIn('"/pl/latarnik"', prepare)
        self.assertIn('"/latarnik"', prepare)
        self.assertIn("pendingAppLink", kotlin)
        self.assertIn("takeAppLink", kotlin)
        self.assertIn("intent.data = null", kotlin)
        self.assertIn("queryParameterNames", kotlin)
        self.assertIn('"takeAppLink"', rust_plugin)

    def test_a_refused_latarnik_intent_is_reported_rather_than_dropped(self):
        kotlin = read("src-tauri/android-push/SignalPushPlugin.kt")
        rust_plugin = read("src-tauri/src/push_plugin.rs")
        # An intent addressed to a Latarnik path is always answered: accepted,
        # or refused out loud. Silence is reserved for ordinary launches.
        self.assertIn("private fun isLatarnikIntent(", kotlin)
        self.assertIn("pendingAppLinkRejected", kotlin)
        self.assertIn('result.put("rejected", rejected)', kotlin)
        self.assertIn("rejected: bool", rust_plugin)
        self.assertIn('return Err("rejected_latarnik_app_link".to_owned())', rust_plugin)
        # The Android capability grammar must not be wider than the Rust one.
        self.assertNotIn("isLetterOrDigit()", kotlin)
        self.assertIn("private fun isInviteChar(character: Char): Boolean", kotlin)
        self.assertIn("character in 'A'..'Z' || character in 'a'..'z' || character in '0'..'9'", kotlin)

    def test_a_fresh_beacon_session_has_no_push_endpoint_to_reconcile(self):
        native = read("src-tauri/src/commands/beacon.rs")
        body = native.split("async fn persist_exchanged_beacon(", 1)[1].split("#[tauri::command]", 1)[0]
        self.assertIn("push_enabled: false,", body)
        self.assertIn("push_last_sync_ok: true,", body)

if __name__ == "__main__":
    unittest.main()
