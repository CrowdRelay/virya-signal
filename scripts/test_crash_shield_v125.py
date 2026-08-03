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
        self.assertIn("native_crash_report", BRIDGE)
        self.assertIn("acknowledge_native_crash", BRIDGE)
        self.assertIn("report('native-panic', previous)", BRIDGE)
        self.assertLess(
            BRIDGE.index("report('native-panic', previous)"),
            BRIDGE.index("core.invoke('acknowledge_native_crash')"),
        )
        self.assertIn("#virya-runtime-failure", STYLES)

    def test_native_panics_survive_process_restart(self):
        self.assertIn("NATIVE_CRASH_REPORT_PATH", NATIVE)
        self.assertIn("write_native_crash_report", NATIVE)
        self.assertIn("fn native_crash_report(", NATIVE)
        self.assertIn("fn acknowledge_native_crash(", NATIVE)
        self.assertIn("file.sync_all()", NATIVE)
        self.assertIn("std::fs::rename(&temporary, path)", NATIVE)
        self.assertNotIn("handle.emit(\"virya-native-crash\"", NATIVE)

    def test_onboarding_debug_list_is_gone(self):
        self.assertNotIn("signal-onboarding-progress", APP)


if __name__ == "__main__":
    unittest.main()
