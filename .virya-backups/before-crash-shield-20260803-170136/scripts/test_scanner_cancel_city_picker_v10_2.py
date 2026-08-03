import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
APP = (ROOT / "src/app.rs").read_text(encoding="utf-8")
BRIDGE = (ROOT / "src/bridge.rs").read_text(encoding="utf-8")
STYLES = (ROOT / "styles.css").read_text(encoding="utf-8")


class SolidScannerAndCityFlowV102(unittest.TestCase):
    def test_cancel_returns_before_native_teardown(self):
        self.assertIn("resolveCancel?.(VIRYA_SCAN_CANCELLED)", BRIDGE)
        self.assertIn("overlay.cancelPromise", BRIDGE)
        self.assertNotIn("await scanner.cancel?.()", BRIDGE)
        self.assertLess(
            BRIDGE.index("resolveCancel?.(VIRYA_SCAN_CANCELLED)"),
            BRIDGE.index("void nativeCancel()"),
        )

    def test_city_collection_is_bounded_reactive_state(self):
        start = APP.index("fn FanAccess(")
        end = APP.index("fn FanApp(", start)
        fan = APP[start:end]
        self.assertNotIn("PublicLoadingState", fan)
        self.assertNotIn("Skeleton", fan)
        self.assertNotIn("collect_view()", fan)
        self.assertIn("Vec::<bridge::PublicCity>::new()", fan)
        self.assertIn("open_public_city_picker(", fan)
        self.assertIn("filtered_public_cities", fan)
        self.assertIn("city_picker_alive", fan)
        self.assertIn("bridge::load_public_cities(API_BASE)", APP)
        self.assertIn("StoredValue::new(Arc::new(AtomicBool::new(true)))", APP)
        self.assertEqual(APP.count("fn normalized_city_query("), 1)
        self.assertEqual(APP.count("fn filtered_public_cities("), 1)
        self.assertIn("alive.load(Ordering::Acquire)", APP)

    def test_city_picker_is_typed_bounded_and_component_owned(self):
        self.assertIn("'public_cities'", BRIDGE)
        self.assertIn(".slice(0, 250)", BRIDGE)
        self.assertIn(".take(40)", APP)
        self.assertIn('role="dialog"', APP)
        self.assertIn("city_picker_open", APP)
        self.assertIn("z-index: 140", STYLES)
        self.assertNotIn("list.replaceChildren(fragment)", BRIDGE)
        self.assertNotIn("viryaOpenCityPicker", BRIDGE)
        self.assertNotIn("event.target === overlay", BRIDGE)


if __name__ == "__main__":
    unittest.main()
