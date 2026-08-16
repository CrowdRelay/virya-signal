from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[1]


class LatarnikStaffShortcutContract(unittest.TestCase):
    def test_staff_settings_link_to_website_and_existing_commerce_control_center(self) -> None:
        source = (ROOT / "src/app/operator/commerce_settings.rs").read_text(encoding="utf-8")
        self.assertIn('https://virya.music/?source=signal-staff-settings', source)
        self.assertIn('https://virya.music/staff/commerce/?source=signal-staff-latarnik', source)
        self.assertIn('>"Virya.music"</button>', source)
        self.assertIn('>"Latarnik · wydania i network"</button>', source)
        self.assertEqual(source.count('signal-staff-latarnik'), 1)

    def test_fan_settings_link_directly_to_virya_music(self) -> None:
        source = (ROOT / "src/app/fan/wallet.rs").read_text(encoding="utf-8")
        self.assertIn('https://virya.music/?source=signal-app-settings', source)
        self.assertIn('label="Virya.music"', source)
        self.assertEqual(source.count('signal-app-settings'), 1)


if __name__ == "__main__":
    unittest.main()
