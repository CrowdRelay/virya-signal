import struct
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
APP = (ROOT / "src/app.rs").read_text(encoding="utf-8")
NATIVE = (ROOT / "src-tauri/src/lib.rs").read_text(encoding="utf-8")
API = (ROOT / "src-tauri/src/api.rs").read_text(encoding="utf-8")
STYLES = (ROOT / "styles.css").read_text(encoding="utf-8")


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
        self.assertIn("NIE MA MOJEGO MIASTA", APP)
        self.assertIn("150 km", APP)

    def test_three_bar_identity_is_centered(self):
        self.assertIn(".signal-logo { align-items: center; }", STYLES)
        raw = (ROOT / "src-tauri/icons/icon.png").read_bytes()
        self.assertEqual(raw[:8], b"\x89PNG\r\n\x1a\n")
        self.assertEqual(struct.unpack(">II", raw[16:24]), (1024, 1024))
        svg = (ROOT / "src-tauri/icons/virya-signal.svg").read_text(encoding="utf-8")
        self.assertIn('id="signal-bars"', svg)
        self.assertEqual(svg.count("<rect"), 4)  # background + three bars


if __name__ == "__main__":
    unittest.main()
