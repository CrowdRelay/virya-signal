from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[1]
BRIDGE = (ROOT / "src/bridge.rs").read_text(encoding="utf-8")
BOOT = (ROOT / "boot.js").read_text(encoding="utf-8")
NATIVE = (ROOT / "src-tauri/src/lib.rs").read_text(encoding="utf-8")
README = (ROOT / "README.md").read_text(encoding="utf-8")


class CrashLedgerV14(unittest.TestCase):
    def test_operation_is_persisted_without_arguments_or_capability_url(self):
        self.assertIn("VIRYA_OPERATION_STORAGE_KEY", BRIDGE)
        self.assertIn("viryaPersistOperation(operation)", BRIDGE)
        self.assertIn("viryaStorageRemove(VIRYA_OPERATION_STORAGE_KEY)", BRIDGE)
        self.assertIn("location.origin}${window.location.pathname", BRIDGE)
        self.assertNotIn("window.location.href", BRIDGE)
        self.assertNotIn("JSON.stringify(args)", BRIDGE)

    def test_abnormal_foreground_session_is_recovered(self):
        self.assertIn("VIRYA_SESSION_STORAGE_KEY", BOOT)
        self.assertIn("__VIRYA_BOOT_DIAGNOSTIC__", BOOT)
        self.assertIn("unexpected-foreground-termination", BOOT)
        self.assertIn("visibilitychange", BOOT)
        self.assertIn("pagehide", BOOT)
        self.assertIn("viryaRecoverBootDiagnostic", BRIDGE)

    def test_native_report_is_atomic_and_deleted_only_after_ack(self):
        self.assertIn("native_crash_report", NATIVE)
        self.assertIn("acknowledge_native_crash", NATIVE)
        self.assertIn("sync_all", NATIVE)
        self.assertIn("std::fs::rename", NATIVE)
        setup = NATIVE[NATIVE.index(".setup(|app|"):NATIVE.index(".invoke_handler", NATIVE.index(".setup(|app|"))]
        self.assertNotIn("remove_file(&crash_report_path)", setup)
        self.assertIn("native_crash_report,", NATIVE)
        self.assertIn("acknowledge_native_crash,", NATIVE)
        self.assertIn("viryaRecoverNativeCrash", BRIDGE)

    def test_failure_history_is_bounded_and_city_crash_path_stays_removed(self):
        self.assertIn("MAX_RUNTIME_FAILURES", BRIDGE)
        self.assertIn("slice(0, MAX_RUNTIME_FAILURES)", BRIDGE)
        self.assertNotIn("load_public_cities", BRIDGE)
        self.assertNotIn("viryaLoadPublicCities", BRIDGE)

    def test_portfolio_docs_are_linked(self):
        self.assertIn("docs/ARCHITECTURE.md", README)
        self.assertIn("docs/RELIABILITY.md", README)
        self.assertIn("QUALITY.md", README)


if __name__ == "__main__":
    unittest.main()
