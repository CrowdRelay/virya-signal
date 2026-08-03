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

if __name__ == "__main__":
    unittest.main()
