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
        self.assertIn("virya-scanner-cancel", BRIDGE)
        self.assertIn("VIRYA_SCAN_CANCELLED", BRIDGE)
        self.assertIn("Result<Option<String>, String>", BRIDGE)

    def test_cancel_is_silent_for_both_scan_callers(self):
        self.assertEqual(APP.count("Ok(None) => {}"), 2)
        self.assertEqual(APP.count("match bridge::scan_qr().await"), 2)

    def test_city_fetch_never_updates_a_native_select(self):
        start = APP.index("fn FanAccess(")
        end = APP.index("fn FanApp(", start)
        fan = APP[start:end]
        city_start = fan.index('<Show when=move || !custom_city.get()>')
        city_end = fan.index('<button class="text-button city-toggle"', city_start)
        city = fan[city_start:city_end]
        self.assertNotIn("<select", city)
        self.assertNotIn("<option", city)
        self.assertIn('class="city-choice-list"', city)
        self.assertIn("stable_public_cities(public).into_iter().map", city)
        self.assertIn("selected_city_name", fan)

    def test_overlay_and_city_buttons_have_touch_targets(self):
        self.assertIn("#virya-scanner-overlay", STYLES)
        self.assertIn("#virya-scanner-cancel", STYLES)
        self.assertIn(".city-choice-list button", STYLES)
        self.assertIn("min-height: 52px", STYLES)


if __name__ == "__main__":
    unittest.main()
