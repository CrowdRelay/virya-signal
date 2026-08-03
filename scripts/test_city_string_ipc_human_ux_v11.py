from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[1]


class CityStringIpcAndHumanUxV11(unittest.TestCase):
    def test_onboarding_explains_value_without_reactive_city_picker(self):
        app = (ROOT / "src/app.rs").read_text(encoding="utf-8")
        self.assertNotIn("signal-onboarding-progress", app)
        self.assertIn("koncerty blisko Ciebie", app)
        self.assertNotIn("SZUKAJ MIASTA", app)
        self.assertIn("city-stable-entry", app)
        self.assertIn("Wpisz miejscowość ręcznie", app)
        self.assertIn("Nie musisz zbierać wszystkiego ani jeździć po kraju", app)

if __name__ == "__main__":
    unittest.main()
