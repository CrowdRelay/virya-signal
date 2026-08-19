#!/usr/bin/env python3
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

class SynesthesiaMobileIntegrationTests(unittest.TestCase):
    def test_fan_home_has_one_canonical_but_demoted_synesthesia_card(self):
        home = (ROOT / "src/app/fan_home.rs").read_text(encoding="utf-8")
        shell = (ROOT / "src/app/fan/shell.rs").read_text(encoding="utf-8")
        self.assertEqual(home.count("synesthesia-home-card"), 1)
        self.assertIn("Synesthesia is a side album experiment, not a primary Home CTA", home)
        self.assertIn("<Show when=move || show_synesthesia>", home)
        self.assertIn("let show_synesthesia = synesthesia.started || synesthesia.completed;", home)
        self.assertNotIn("FanSynesthesiaCard", shell)
        self.assertNotIn("synesthesia-entry-card", shell)

    def test_canonical_card_resumes_local_journey(self):
        home = (ROOT / "src/app/fan_home.rs").read_text(encoding="utf-8")
        self.assertIn("https://synesthesia.virya.music/?source=signal-app&resume=1", home)
        self.assertIn('if synesthesia.started { tr("open_synesthesia") } else { tr("enter_synesthesia") }', home)

if __name__ == "__main__":
    unittest.main()
