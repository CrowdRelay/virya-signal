import re
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
APP = (ROOT / "src/app.rs").read_text(encoding="utf-8")
BRIDGE = (ROOT / "src/bridge.rs").read_text(encoding="utf-8")
STYLES = (ROOT / "styles.css").read_text(encoding="utf-8")


class ScannerCancelAndCityPickerV101(unittest.TestCase):
    def test_scanner_is_windowed_and_cancellable(self):
        self.assertIn("windowed: true", BRIDGE)
        self.assertIn("scanner.cancel", BRIDGE)
        self.assertIn("VIRYA_SCAN_CANCELLED", BRIDGE)
        self.assertIn("Result<Option<String>, String>", BRIDGE)

    def test_cancel_is_silent_for_both_scan_callers(self):
        matches = list(re.finditer(r"match bridge::scan_qr\(\)\.await", APP))
        self.assertEqual(len(matches), 2)
        for match in matches:
            block = APP[match.start():match.start() + 300]
            self.assertIn("Ok(None) => {}", block)

    def test_city_never_updates_a_leptos_collection(self):
        start = APP.index("fn FanAccess(")
        end = APP.index("fn FanApp(", start)
        fan = APP[start:end]
        self.assertNotIn("<select", fan)
        self.assertNotIn("stable_public_cities", fan)
        self.assertNotIn("refresh_public_cities", fan)
        self.assertNotIn("PublicLoadingState", fan)
        self.assertIn("bridge::pick_public_city(API_BASE)", fan)

    def test_overlays_have_touch_targets(self):
        self.assertIn("#virya-scanner-overlay", STYLES)
        self.assertIn("#virya-city-picker-list button", STYLES)
        self.assertIn("min-height: 52px", STYLES)


if __name__ == "__main__":
    unittest.main()
