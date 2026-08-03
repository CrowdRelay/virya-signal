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

    def test_city_collection_never_enters_reactive_rust_state(self):
        start = APP.index("fn FanAccess(")
        end = APP.index("fn FanApp(", start)
        fan = APP[start:end]
        self.assertNotIn("PublicLoadingState", fan)
        self.assertNotIn("Skeleton", fan)
        self.assertNotIn("collect_view()", fan)
        self.assertIn("bridge::pick_public_city(API_BASE)", fan)

    def test_city_picker_is_imperative_and_bounded(self):
        self.assertIn("'public_cities'", BRIDGE)
        self.assertIn(".slice(0, 250)", BRIDGE)
        self.assertIn(".slice(0, 30)", BRIDGE)
        self.assertIn("list.replaceChildren(fragment)", BRIDGE)
        self.assertIn("event.key === 'Escape'", BRIDGE)
        self.assertIn("event.target === overlay", BRIDGE)
        self.assertIn("z-index: 120", STYLES)


if __name__ == "__main__":
    unittest.main()
