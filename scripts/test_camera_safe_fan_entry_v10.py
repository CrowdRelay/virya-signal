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
    def test_camera_permission_is_requested_before_scan(self):
        function = BRIDGE[BRIDGE.index("export async function viryaScanQr") :]
        self.assertIn("await viryaEnsureCameraPermission(scanner)", function)
        self.assertLess(
            BRIDGE.index("await viryaEnsureCameraPermission(scanner)"),
            BRIDGE.index("await scanner.scan"),
        )
        self.assertIn("checkPermissions", BRIDGE)
        self.assertIn("requestPermissions", BRIDGE)

    def test_mobile_capability_allows_permission_commands(self):
        self.assertIn("barcode-scanner:default", CAPABILITY["permissions"])
        self.assertIn("android", CAPABILITY["platforms"])

    def test_fan_entry_has_no_automatic_network_request(self):
        start = APP.index("fn FanPortal(")
        end = APP.index("fn AccessLoader(", start)
        block = APP[start:end]
        self.assertNotIn("refresh_public_cities(", block)
        self.assertIn("PublicLoadingState::default()", block)

    def test_custom_city_is_the_local_first_default(self):
        start = APP.index("fn FanAccess(")
        end = APP.index("fn FanApp(", start)
        block = APP[start:end]
        self.assertIn("let custom_city = RwSignal::new(true);", block)
        self.assertIn("let cities_started = RwSignal::new(false);", block)
        self.assertIn("on:click=toggle_city_mode", block)
        self.assertIn("refresh_public_cities(public, public_loading, error);", block)


if __name__ == "__main__":
    unittest.main()
