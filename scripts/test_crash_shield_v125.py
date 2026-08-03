from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[1]
APP = (ROOT / "src/app.rs").read_text(encoding="utf-8")
BRIDGE = (ROOT / "src/bridge.rs").read_text(encoding="utf-8")
NATIVE = (ROOT / "src-tauri/src/lib.rs").read_text(encoding="utf-8")
STYLES = (ROOT / "styles.css").read_text(encoding="utf-8")


class CrashShieldV125(unittest.TestCase):
    def test_city_onboarding_never_loads_remote_collection(self):
        start = APP.index("fn FanAccess(")
        end = APP.index("fn FanApp(", start)
        block = APP[start:end]
        self.assertNotIn("Vec::<bridge::PublicCity>::new()", block)
        self.assertNotIn("bridge::load_public_cities", block)
        self.assertNotIn("open_public_city_picker", block)
        self.assertNotIn("<For", block)
        self.assertNotIn("city_picker_", block)
        self.assertIn("request_city", block)
        self.assertIn("city-stable-entry", block)

    def test_webview_failures_are_persistent_and_copyable(self):
        self.assertIn("VIRYA_FAILURE_STORAGE_KEY", BRIDGE)
        self.assertIn("KOPIUJ RAPORT", BRIDGE)
        self.assertIn("unhandledrejection", BRIDGE)
        self.assertIn("virya-native-crash", BRIDGE)
        self.assertIn("#virya-runtime-failure", STYLES)

    def test_native_panics_survive_process_restart(self):
        self.assertIn("NATIVE_CRASH_REPORT_PATH", NATIVE)
        self.assertIn("write_native_crash_report", NATIVE)
        self.assertIn("handle.emit(\"virya-native-crash\"", NATIVE)

    def test_onboarding_debug_list_is_gone(self):
        self.assertNotIn("signal-onboarding-progress", APP)


if __name__ == "__main__":
    unittest.main()
