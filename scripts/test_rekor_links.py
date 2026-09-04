from rust_source_tree import read_rust_module
import pathlib
import unittest

from source_tree import read_app_source

ROOT = pathlib.Path(__file__).resolve().parents[1]


class RekorLinkTests(unittest.TestCase):
    def test_draw_models_accept_slug_without_breaking_legacy_payloads(self):
        web = (ROOT / "src/models.rs").read_text()
        native = read_rust_module(ROOT, "src-tauri/src/models.rs")
        self.assertIn("pub slug: String", web)
        self.assertIn("#[serde(default)]", web)
        self.assertIn('default, deserialize_with = "deserialize_string_or_bytes"', native)

    def test_fan_draw_card_opens_public_proof(self):
        app = read_app_source(ROOT)
        # The locale is the fan's, not a hardcoded /pl/: an English fan
        # following a proof link must not land on a Polish page.
        self.assertIn("https://virya.music/{}/dowody/losowania/{}/?source=signal-app", app)
        self.assertIn("i18n::current().code()", app)
        self.assertIn('label=tr("proof")', app)
        self.assertIn("open_external_url", app)


if __name__ == "__main__":
    unittest.main()
