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

if __name__ == "__main__":
    unittest.main()
