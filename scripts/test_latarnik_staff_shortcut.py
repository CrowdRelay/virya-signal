from pathlib import Path
import unittest

from i18n_catalog import load_catalog_pair

ROOT = Path(__file__).resolve().parents[1]


class LatarnikStaffShortcutContract(unittest.TestCase):
    """Both shortcuts must exist, be labelled, and point where they claim to.

    These used to be asserted as exact markup (`>"Virya.music"</button>`),
    which pinned the shape of the row rather than the contract. Turning the
    settings rows into labelled, described controls broke every one of those
    assertions without changing a single destination. What matters is the URL,
    the label the user reads, and the fact that the Latarnik shortcut appears
    exactly once — so that is what is checked.
    """

    @classmethod
    def setUpClass(cls) -> None:
        cls.pl, cls.en = load_catalog_pair(ROOT)

    def test_staff_settings_link_to_website_and_existing_commerce_control_center(self) -> None:
        source = (ROOT / "src/app/operator/commerce_settings.rs").read_text(encoding="utf-8")
        self.assertIn('https://virya.music/?source=signal-staff-settings', source)
        self.assertIn('https://virya.music/staff/commerce/?source=signal-staff-latarnik', source)
        self.assertIn('"Virya.music"', source)
        self.assertIn('tr("staff_latarnik_panel")', source)
        self.assertEqual(self.pl["staff_latarnik_panel"], "Latarnik · wydania i network")
        self.assertEqual(source.count('signal-staff-latarnik'), 1)

    def test_fan_settings_link_directly_to_virya_music(self) -> None:
        source = (ROOT / "src/app/fan/wallet.rs").read_text(encoding="utf-8")
        self.assertIn('https://virya.music/?source=signal-app-settings', source)
        self.assertIn('"Virya.music"', source)
        self.assertEqual(source.count('signal-app-settings'), 1)


if __name__ == "__main__":
    unittest.main()
