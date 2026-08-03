from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[1]
APP = (ROOT / "src/app.rs").read_text(encoding="utf-8")


class CrashShieldV1251(unittest.TestCase):
    def test_old_contracts_cannot_require_removed_city_ui(self):
        scripts = "\n".join(
            path.read_text(encoding="utf-8")
            for path in sorted((ROOT / "scripts").glob("test_*.py"))
            if path.name != Path(__file__).name
        )
        self.assertNotIn('assertIn("signal-onboarding-progress"', scripts)
        self.assertNotIn("assertIn('signal-onboarding-progress'", scripts)
        bad_scope_contract = "assertIn(" + repr('invalidate_latest("public:fan-access:")')
        self.assertNotIn(bad_scope_contract, scripts)

    def test_fan_access_has_no_remote_city_collection(self):
        start = APP.index("fn FanAccess(")
        end = APP.index("fn FanApp(", start)
        block = APP[start:end]
        self.assertIn("city-stable-entry", block)
        self.assertIn("request_city", block)
        self.assertNotIn("PublicCity", block)
        self.assertNotIn("load_public_cities", block)
        self.assertNotIn("city_picker_", block)
        self.assertNotIn("<For", block)


if __name__ == "__main__":
    unittest.main()
