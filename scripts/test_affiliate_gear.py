from pathlib import Path
import re
import unittest

ROOT = Path(__file__).resolve().parents[1]
AFFILIATE = ROOT / "src/app/fan/affiliate.rs"
AFFILIATE_I18N = ROOT / "src/i18n/affiliate.rs"
MERCH = ROOT / "src/app/fan/merch.rs"
INDEX = ROOT / "index.html"
POLISH_DIACRITICS = re.compile(r"[ąćęłńóśźżĄĆĘŁŃÓŚŹŻ]")


class AffiliateGearContracts(unittest.TestCase):
    def test_thomann_tracking_is_signal_scoped(self):
        source = AFFILIATE.read_text(encoding="utf-8")
        self.assertIn("offid=1", source)
        self.assertIn("affid=4979", source)
        self.assertIn("subid=signal&subid2=gear", source)
        self.assertIn("subid=signal&subid2=shop", source)
        self.assertEqual(source.count("https://www.thomann.pl/"), 2)
        self.assertNotIn("clickfi.re", source)

    def test_affiliate_copy_is_localized_and_transparent(self):
        ui = AFFILIATE.read_text(encoding="utf-8")
        copy = AFFILIATE_I18N.read_text(encoding="utf-8")
        self.assertIsNone(POLISH_DIACRITICS.search(ui))
        self.assertIn("Linki afiliacyjne", copy)
        self.assertIn("Affiliate links", copy)
        self.assertIn("bez dodatkowych kosztów", copy)
        self.assertIn("at no extra cost", copy)
        self.assertNotIn("3.5%", copy)
        self.assertNotIn("3,5%", copy)

    def test_affiliate_section_stays_inside_merch_and_uses_its_own_stylesheet(self):
        merch = MERCH.read_text(encoding="utf-8")
        index = INDEX.read_text(encoding="utf-8")
        self.assertIn('include!("affiliate.rs");', merch)
        self.assertIn("<FanAffiliateGear error=error />", merch)
        self.assertIn('data-trunk rel="css" href="affiliate.css"', index)


if __name__ == "__main__":
    unittest.main()
