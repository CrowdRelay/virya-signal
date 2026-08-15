#!/usr/bin/env python3
import unittest
from pathlib import Path
ROOT = Path(__file__).resolve().parents[1]
FAN = (ROOT / 'src/app/fan.rs').read_text()
SHELL = (ROOT / 'src/app/fan/shell.rs').read_text()
PL = (ROOT / 'src/i18n/pl.rs').read_text()
EN = (ROOT / 'src/i18n/en.rs').read_text()

class LatarnikSignalEntryContract(unittest.TestCase):
    def test_portal_is_available_before_and_after_fan_login(self):
        for source in (FAN, SHELL):
            self.assertIn('https://virya.music/pl/latarnik/', source)
            self.assertIn('https://virya.music/latarnik/', source)
            self.assertIn('open_external_url', source)
        self.assertIn('latarnik-entry', FAN)
        self.assertIn('latarnik_zone', SHELL)

    def test_copy_exists_in_both_languages(self):
        self.assertIn('"latarnik_zone"', PL)
        self.assertIn('"latarnik_zone"', EN)
        self.assertIn('"latarnik_short_pitch"', PL)
        self.assertIn('"latarnik_short_pitch"', EN)

if __name__ == '__main__': unittest.main()
