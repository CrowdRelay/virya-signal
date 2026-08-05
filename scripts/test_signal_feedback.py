import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]


class SignalFeedbackContracts(unittest.TestCase):
    def test_ui_exposes_feedback_without_profile_fields(self):
        ui = (ROOT / "src/app/mod.rs").read_text()
        native = (ROOT / "src-tauri/src/api/site.rs").read_text()
        commands = (ROOT / "src-tauri/src/commands/misc.rs").read_text()
        self.assertIn("ANONIMOWY FEEDBACK", ui)
        self.assertIn("submit_anonymous_feedback", ui)
        self.assertIn("SignalFeedbackRequest", native)
        self.assertIn("submission_id: Uuid::new_v4()", native)
        self.assertNotIn("email:", native.split("struct SignalFeedbackRequest", 1)[1].split("}", 1)[0])
        self.assertNotIn("token:", native.split("struct SignalFeedbackRequest", 1)[1].split("}", 1)[0])
        self.assertIn("pub(crate) async fn submit_anonymous_feedback", commands)

    def test_bundle_catalog_is_same_origin_and_bounded(self):
        native = (ROOT / "src-tauri/src/api/site.rs").read_text()
        ui = (ROOT / "src/app/mod.rs").read_text()
        self.assertIn("https://virya.music/api/merch/inventory", native)
        self.assertIn('Some("virya.music" | "www.virya.music")', native)
        self.assertIn("MAX_BUNDLES", native)
        self.assertIn("MAX_VARIANTS", native)
        self.assertIn("FanMerchBundleCatalog", ui)
        self.assertIn("Result<SignalMerchBundleCatalog, AppError>", native)
        self.assertNotIn("Result<serde_json::Value, AppError>", native)


if __name__ == "__main__":
    unittest.main()
