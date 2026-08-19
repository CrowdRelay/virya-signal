#!/usr/bin/env python3
"""Keep Signal's typed next-best-action contract hermetic in standalone CI."""

from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[1]
BACKEND = ROOT.parent / "crowdrelay/crates/crowdrelay-api/src/fan_context.rs"


class NextBestActionV2Contract(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.ui = (ROOT / "src/app/fan_home.rs").read_text(encoding="utf-8")
        cls.contract = (
            ROOT / "crates/virya-signal-contracts/src/fan.rs"
        ).read_text(encoding="utf-8")

    def test_signal_uses_typed_recommended_targets(self) -> None:
        self.assertIn("snapshot.recommended.as_ref()", self.ui)
        self.assertIn("FanTarget::parse", self.ui)
        self.assertIn('strip_prefix("event=")', self.contract)
        self.assertIn("not_event", self.contract)

    def test_typed_target_slug_is_bounded_and_sanitized(self) -> None:
        self.assertIn("fn event_slug(value: &str)", self.contract)
        self.assertIn("value.len() > 128", self.contract)
        self.assertIn("byte.is_ascii_lowercase()", self.contract)
        self.assertIn("byte.is_ascii_digit()", self.contract)
        self.assertIn("matches!(byte, b'-' | b'_')", self.contract)

    def test_backend_parity_when_ecosystem_checkout_is_available(self) -> None:
        # Standalone Signal CI intentionally checks out only this repository.
        # Ecosystem worktrees place CrowdRelay next to Signal; when it is there,
        # keep the cross-repo recommendation contract honest as an extra check.
        if not BACKEND.exists():
            return

        backend = BACKEND.read_text(encoding="utf-8")
        for token in (
            "live_admission_ready",
            "ticket_sale_active",
            "FanRecommendedActionDetail",
            "recommended_action_detail",
        ):
            with self.subTest(token=token):
                self.assertIn(token, backend)
        self.assertNotIn("synesthesia_incomplete", backend)
        self.assertNotIn('action("continue_synesthesia"', backend)


if __name__ == "__main__":
    unittest.main()
