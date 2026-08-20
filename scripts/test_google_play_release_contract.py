from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[1]


class GooglePlayReleaseContractTests(unittest.TestCase):
    def test_initial_production_release_is_guarded_and_staged(self):
        workflow = (ROOT / ".github/workflows/android-play.yml").read_text()
        for token in (
            "play_track == 'production'",
            'play_status="inProgress"',
            'user_fraction="0.10"',
            "production-requires-wif",
            "check-live-app-links.py",
            "environment:\n      name: ${{ needs.prepare.outputs.play_track == 'production' && 'production' || 'store-testing' }}",
            "whatsNewDirectory: distribution/whatsnew",
        ):
            self.assertIn(token, workflow)

    def test_rollout_promotion_is_manual_wif_only_and_forward_bounded(self):
        workflow = (ROOT / ".github/workflows/android-play-promote.yml").read_text()
        script = (ROOT / "scripts/google_play_rollout.py").read_text()
        self.assertIn("workflow_dispatch:", workflow)
        self.assertIn('options: ["25", "50", "100", "halt"]', workflow)
        self.assertIn("GOOGLE_PLAY_WIF_PROVIDER", workflow)
        self.assertNotIn("GOOGLE_PLAY_SERVICE_ACCOUNT_JSON", workflow)
        self.assertIn("check-live-app-links.py", workflow)
        self.assertIn("CROWDRELAY_READINESS=PASS", workflow)
        self.assertIn("refusing non-forward rollout change", script)
        self.assertIn("multiple active staged releases found", script)
        self.assertIn('"completed"', script)
        self.assertIn('"halted"', script)

    def test_localized_release_notes_are_present_and_bounded(self):
        for locale in ("pl-PL", "en-US"):
            path = ROOT / "distribution/whatsnew" / f"whatsnew-{locale}"
            text = path.read_text().strip()
            self.assertTrue(text)
            self.assertLessEqual(len(text), 500)

    def test_release_runbook_exists(self):
        runbook = (ROOT / "docs/google-play-production-release.md").read_text()
        for token in (
            "10% staged rollout",
            "10% → 25% → 50% → 100%",
            "Data safety",
            "Play App Signing",
            "GOOGLE_PLAY_WIF_PROVIDER",
        ):
            self.assertIn(token, runbook)


if __name__ == "__main__":
    unittest.main()
