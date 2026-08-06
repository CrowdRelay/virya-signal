import pathlib
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]


class RekorLinkTests(unittest.TestCase):
    def test_draw_models_accept_slug_without_breaking_legacy_payloads(self):
        web = (ROOT / "src/models.rs").read_text()
        native = (ROOT / "src-tauri/src/models.rs").read_text()
        self.assertIn("pub slug: String", web)
        self.assertIn("#[serde(default)]", web)
        self.assertIn('default, deserialize_with = "deserialize_string_or_bytes"', native)

    def test_fan_draw_card_opens_public_proof(self):
        app = (ROOT / "src/app/mod.rs").read_text()
        self.assertIn("/pl/dowody/losowania/{}/?source=signal-app", app)
        self.assertIn('label=tr("dowod")', app)
        self.assertIn("open_external_url", app)


if __name__ == "__main__":
    unittest.main()
