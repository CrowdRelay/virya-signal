import re
import subprocess
import unittest
from pathlib import Path

from i18n_catalog import load_catalog_pair

ROOT = Path(__file__).resolve().parents[1]
DIACRITICS = re.compile(r"[ąćęłńóśźżĄĆĘŁŃÓŚŹŻ]")
KEY_PATTERN = re.compile(r'^\s*"([^"]+)"\s*=>', re.MULTILINE)
PLACEHOLDER_PATTERN = re.compile(r"\{([A-Za-z0-9_]+)\}")


class I18nContracts(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.pl, cls.en = load_catalog_pair(ROOT)

    def test_rust_catalogs_have_exactly_the_manifest_keys(self):
        expected = set(self.pl)
        self.assertEqual(set(self.en), expected)
        for language in ("pl", "en"):
            source = (ROOT / f"src/i18n/{language}.rs").read_text()
            keys = KEY_PATTERN.findall(source)
            self.assertEqual(len(keys), len(set(keys)), f"duplicate {language} key")
            self.assertEqual(set(keys), expected)

    def test_placeholders_match_between_languages(self):
        for key in self.pl:
            self.assertEqual(
                set(PLACEHOLDER_PATTERN.findall(self.pl[key])),
                set(PLACEHOLDER_PATTERN.findall(self.en[key])),
                key,
            )

    def test_native_core_reuses_the_same_static_catalogs(self):
        source = (ROOT / "src-tauri/src/i18n/mod.rs").read_text()
        self.assertIn('../../../src/i18n/pl.rs', source)
        self.assertIn('../../../src/i18n/en.rs', source)
        self.assertIn('AtomicU8', source)
        self.assertNotIn('HashMap', source)

    def test_selected_language_crosses_the_first_native_ipc(self):
        bridge = (ROOT / "src/bridge.rs").read_text()
        launcher = (ROOT / "src-tauri/src/commands/misc.rs").read_text()
        self.assertIn('locale: i18n::current().code()', bridge)
        self.assertIn('i18n::set_language(&locale)', launcher)

    def test_boot_catalog_is_generated_and_valid_javascript(self):
        before = (ROOT / "boot-i18n.js").read_text()
        subprocess.run(
            ["python3", str(ROOT / "scripts/generate-boot-i18n.py")],
            check=True,
            cwd=ROOT,
        )
        self.assertEqual((ROOT / "boot-i18n.js").read_text(), before)
        subprocess.run(["node", "--check", "boot-i18n.js"], check=True, cwd=ROOT)

    def test_runtime_copy_is_not_hardcoded_in_polish(self):
        for relative in ("src/app/mod.rs", "src/app/area.rs", "src/bridge.rs", "boot.js", "index.html"):
            self.assertIsNone(DIACRITICS.search((ROOT / relative).read_text()), relative)
        for path in sorted((ROOT / "src-tauri/src").rglob("*.rs")):
            runtime = path.read_text().split("#[cfg(test)]", 1)[0]
            self.assertIsNone(DIACRITICS.search(runtime), path.relative_to(ROOT))

    def test_language_switch_is_present_for_fan_and_staff(self):
        ui = (ROOT / "src/app/mod.rs").read_text()
        self.assertGreaterEqual(ui.count("<LanguageSwitch />"), 2)
        self.assertIn("Language::Pl", ui)
        self.assertIn("Language::En", ui)
        self.assertIn("virya:language:v1", (ROOT / "src/i18n/mod.rs").read_text())

    def test_catalog_identifiers_are_english_ascii_snake_case(self):
        for key in self.en:
            self.assertRegex(key, r"^[a-z][a-z0-9_]*$")
        legacy_polish_ids = {
            "nie_udao_sie_zapisac_miasta_message",
            "nowa_wiadomosc_nie_zostaa_wysana_bo_poprzedni_kod",
            "nie_udao_sie_odswiezyc_value_zamowien_pozostae_bilety",
        }
        self.assertTrue(legacy_polish_ids.isdisjoint(self.en))


if __name__ == "__main__":
    unittest.main()
