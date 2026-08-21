#!/usr/bin/env python3
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SHELL = (ROOT / "src/app/fan/shell.rs").read_text(encoding="utf-8")
SUPPORT = (ROOT / "src/app/support.rs").read_text(encoding="utf-8")


class FanRefreshStateContract(unittest.TestCase):
    def test_full_refresh_marks_every_requested_section_loaded(self):
        block = SHELL.split("let generation = refresh_requested.get();", 1)[1].split(
            "Effect::new(move |_| {\n        if tab.get() != FanTab::Events", 1
        )[0]
        self.assertNotIn("loaded.set(FanLoadedState::default())", block)
        for field in (
            "home",
            "referral",
            "events",
            "interests",
            "merch",
            "admission_pass",
            "wallets",
            "area",
        ):
            self.assertIn(f"{field}: true", block)
        for refresh in (
            "refresh_fan_home",
            "refresh_fan_parts",
            "refresh_fan_merch",
            "refresh_fan_merch_bundles",
            "refresh_wallets",
            "refresh_fan_area",
        ):
            self.assertIn(refresh, block)

    def test_partial_dashboard_refresh_owns_only_child_loading_flags(self):
        block = SUPPORT.split("fn refresh_fan_parts(", 1)[1].split(
            "fn refresh_fan_events(", 1
        )[0]
        self.assertNotIn("FanLoadingState::all()", block)
        for refresh in (
            "refresh_fan_events",
            "refresh_fan_referral",
            "refresh_fan_interests",
            "refresh_fan_admission_pass",
        ):
            self.assertIn(refresh, block)


if __name__ == "__main__":
    unittest.main()
