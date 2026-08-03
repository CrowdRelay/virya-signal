import hashlib
import struct
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
APP = (ROOT / "src/app.rs").read_text(encoding="utf-8")
NATIVE = (ROOT / "src-tauri/src/lib.rs").read_text(encoding="utf-8")
API = (ROOT / "src-tauri/src/api.rs").read_text(encoding="utf-8")
STYLES = (ROOT / "styles.css").read_text(encoding="utf-8")
APPROVED_ICON_SHA256 = "a8db1cdc69decb1e1cf1774a484cb02b8101a40da191d9374613060a663fce3a"


class PrincipalOnboardingContract(unittest.TestCase):
    def test_shared_access_loader_survives_fan_block_replacement(self):
        self.assertEqual(APP.count("fn AccessLoader("), 1)
        self.assertEqual(APP.count("<AccessLoader "), 2)
        self.assertIn('label="SPRAWDZAM BEZPIECZNY SEJF"', APP)
        self.assertIn('label="SPRAWDZAM TWÓJ SYGNAŁ"', APP)

    def test_registration_route_has_no_async_keyed_event_strip(self):
        start = APP.index("fn FanAccess(")
        end = APP.index("fn FanApp(", start)
        block = APP[start:end]
        self.assertNotIn("PublicEventStrip", block)
        self.assertNotIn("refresh_public_events", block)
        self.assertIn('invalidate_latest("public:fan-access:")', APP)

    def test_native_core_sanitizes_keyed_lists(self):
        self.assertIn("sanitize_public_events", API)
        self.assertIn("sanitize_public_cities", API)
        self.assertGreaterEqual(API.count("dedup_by"), 2)
        self.assertGreaterEqual(API.count("truncate(MAX_PUBLIC_"), 2)

    def test_pairing_and_custom_city_flows_exist(self):
        self.assertIn("configure_from_pairing", NATIVE)
        self.assertIn("parse_pairing_payload", NATIVE)
        self.assertIn("request_city", NATIVE)
        self.assertIn("WPISZ WŁASNE", APP)
        self.assertIn("bridge::load_public_cities(API_BASE)", APP)
        self.assertIn("open_public_city_picker(", APP)
        self.assertIn("on:click=move |_| {", APP)
        self.assertIn("StoredValue::new(Arc::new(AtomicBool::new(true)))", APP)
        self.assertIn("city_picker_alive.try_read_value()", APP)
        self.assertEqual(APP.count("fn open_public_city_picker("), 1)
        self.assertEqual(APP.count("fn normalized_city_query("), 1)
        self.assertEqual(APP.count("fn filtered_public_cities("), 1)
        self.assertNotIn("Rc<Cell<bool>>", APP)
        self.assertNotIn("let open_city_picker", APP)
        self.assertNotIn("bridge::pick_public_city(API_BASE)", APP)
        self.assertIn("150 km", APP)

    def test_exact_approved_identity_is_installed(self):
        self.assertIn(".signal-logo { align-items: center; }", STYLES)
        icon = ROOT / "src-tauri/icons/icon.png"
        raw = icon.read_bytes()
        self.assertEqual(raw[:8], b"\x89PNG\r\n\x1a\n")
        self.assertEqual(struct.unpack(">II", raw[16:24]), (1024, 1024))
        self.assertEqual(hashlib.sha256(raw).hexdigest(), APPROVED_ICON_SHA256)


if __name__ == "__main__":
    unittest.main()
