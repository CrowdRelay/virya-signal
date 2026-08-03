from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[1]

class CityStringIpcAndHumanUxV11(unittest.TestCase):
    def test_city_payload_crosses_ipc_as_bounded_json_string(self):
        native = (ROOT / "src-tauri/src/lib.rs").read_text()
        bridge = (ROOT / "src/bridge.rs").read_text()
        self.assertIn("Result<String, AppError>", native)
        self.assertIn("serde_json::to_string(&cities)", native)
        self.assertIn("value.length > 512_000", bridge)
        self.assertIn("value = JSON.parse(value)", bridge)
        self.assertNotIn("Result<Vec<CitySignal>, AppError>", native)

    def test_onboarding_explains_value_and_progress(self):
        app = (ROOT / "src/app.rs").read_text()
        self.assertIn("signal-onboarding-progress", app)
        self.assertIn("koncerty blisko Ciebie", app)
        self.assertIn("SZUKAJ MIASTA", app)
        self.assertIn("Nie musisz zbierać wszystkiego ani jeździć po kraju", app)

if __name__ == "__main__":
    unittest.main()
