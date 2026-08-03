import json
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
APP = (ROOT / "src/app.rs").read_text(encoding="utf-8")
BRIDGE = (ROOT / "src/bridge.rs").read_text(encoding="utf-8")
CAPABILITY = json.loads(
    (ROOT / "src-tauri/capabilities/mobile.json").read_text(encoding="utf-8")
)


class CameraAndSafeFanEntryContract(unittest.TestCase):
    def test_camera_permission_is_requested_before_scan_start(self):
        function = BRIDGE[BRIDGE.index("export async function viryaScanQr") :]
        permission = function.index("await viryaEnsureCameraPermission(scanner)")
        scan = function.index("scanner.scan(")
        self.assertLess(permission, scan)
        self.assertIn("checkPermissions", BRIDGE)
        self.assertIn("requestPermissions", BRIDGE)

    def test_mobile_capability_allows_permission_commands(self):
        self.assertIn("barcode-scanner:default", CAPABILITY["permissions"])
        self.assertIn("android", CAPABILITY["platforms"])

    def test_fan_entry_has_no_automatic_city_request(self):
        start = APP.index("fn FanPortal(")
        end = APP.index("fn AccessLoader(", start)
        block = APP[start:end]
        self.assertNotIn("refresh_public_cities(", block)
        self.assertNotIn("PublicLoadingState", block)

    def test_custom_city_is_local_first_and_picker_is_explicit(self):
        start = APP.index("fn FanAccess(")
        end = APP.index("fn FanApp(", start)
        block = APP[start:end]
        self.assertIn("let custom_city = RwSignal::new(true);", block)
        self.assertIn("open_public_city_picker(", block)
        self.assertIn("StoredValue::new(Arc::new(AtomicBool::new(true)))", block)
        self.assertIn("city_picker_alive.try_read_value()", block)
        self.assertIn("city_picker_alive", block)
        self.assertEqual(APP.count("fn open_public_city_picker("), 1)
        self.assertEqual(APP.count("fn normalized_city_query("), 1)
        self.assertEqual(APP.count("fn filtered_public_cities("), 1)
        self.assertIn("bridge::load_public_cities(API_BASE)", APP)
        self.assertIn("alive.load(Ordering::Acquire)", APP)
        self.assertNotIn("let open_city_picker", block)
        self.assertNotIn("Rc<", APP)
        self.assertIn("WPISZ WŁASNE", block)
        self.assertNotIn("bridge::pick_public_city", block)
        self.assertNotIn("refresh_public_cities", block)


if __name__ == "__main__":
    unittest.main()
