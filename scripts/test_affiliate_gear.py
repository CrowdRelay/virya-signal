from pathlib import Path
import re
import unittest

ROOT = Path(__file__).resolve().parents[1]
AFFILIATE = ROOT / "src/app/fan/affiliate.rs"
# Affiliate copy lives in the shared PL/EN catalogs like every other string,
# not in its own Rust module that would ship inside the WASM data section.
AFFILIATE_PL = ROOT / "src/i18n/pl.rs"
AFFILIATE_EN = ROOT / "src/i18n/en.rs"
MERCH = ROOT / "src/app/fan/merch.rs"
INDEX = ROOT / "index.html"
POLISH_DIACRITICS = re.compile(r"[ąćęłńóśźżĄĆĘŁŃÓŚŹŻ]")


class AffiliateGearContracts(unittest.TestCase):
    def test_thomann_tracking_is_signal_scoped(self):
        # Affiliate identifiers moved server-side into CrowdRelay smart links, so
        # the shipped client carries first-party redirect URLs only. Tracking is
        # still Signal-scoped, now through the slug rather than query parameters,
        # and the partner ids can be rotated without releasing a new build.
        source = AFFILIATE.read_text(encoding="utf-8")
        for slug in ("thomann-qc-signal", "thomann-shop-signal"):
            self.assertIn(f"https://signal-api.virya.music/v1/go/{slug}", source)
            self.assertTrue(slug.endswith("-signal"), "smart link must stay Signal-scoped")
        self.assertEqual(source.count("https://signal-api.virya.music/v1/go/"), 2)
        # No affiliate identifier or partner endpoint may be embedded in the client.
        for leaked in ("offid=", "affid=", "subid=", "thomann.pl", "clickfi.re"):
            self.assertNotIn(leaked, source)

    def test_affiliate_copy_is_localized_and_transparent(self):
        ui = AFFILIATE.read_text(encoding="utf-8")
        polish = AFFILIATE_PL.read_text(encoding="utf-8")
        english = AFFILIATE_EN.read_text(encoding="utf-8")
        self.assertIsNone(POLISH_DIACRITICS.search(ui))
        # The component must reach copy through the runtime catalog, never hold it.
        self.assertNotIn("AffiliateGearCopy", ui)
        self.assertIn('tr("affiliate_disclosure")', ui)
        self.assertIn("Linki afiliacyjne", polish)
        self.assertIn("Affiliate links", english)
        self.assertIn("bez dodatkowych kosztów", polish)
        self.assertIn("at no extra cost", english)
        for catalog in (polish, english):
            self.assertNotIn("3.5%", catalog)
            self.assertNotIn("3,5%", catalog)

    def test_affiliate_section_stays_inside_merch_and_uses_its_own_stylesheet(self):
        merch = MERCH.read_text(encoding="utf-8")
        index = INDEX.read_text(encoding="utf-8")
        self.assertIn('include!("affiliate.rs");', merch)
        self.assertIn("<FanAffiliateGear error=error />", merch)
        self.assertIn('data-trunk rel="css" href="affiliate.css"', index)


if __name__ == "__main__":
    unittest.main()
