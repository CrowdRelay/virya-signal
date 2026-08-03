import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
APP = (ROOT / "src/app.rs").read_text(encoding="utf-8")
STYLES = (ROOT / "styles.css").read_text(encoding="utf-8")


class FanCrashContainmentV97(unittest.TestCase):
    def test_registration_has_no_keyed_remote_city_list(self):
        start = APP.index("fn FanAccess(")
        end = APP.index("fn FanApp(", start)
        block = APP[start:end]
        self.assertNotIn("<For", block)
        self.assertIn("stable_public_cities(public).into_iter()", block)

    def test_authenticated_fan_loads_by_active_tab(self):
        start = APP.index("fn FanApp(")
        end = APP.index("fn FanNavButton(", start)
        block = APP[start:end]
        self.assertIn("FanLoadedState::default()", block)
        self.assertIn("match tab.get()", block)
        self.assertIn("FanTab::Signal", block)
        self.assertIn("FanTab::Wallet", block)
        self.assertIn('invalidate_latest("fan:")', block)
        self.assertNotIn(
            "refresh_fan_parts(dashboard, loading, error);",
            block,
        )

    def test_remote_fan_lists_are_stabilized(self):
        for helper in (
            "stable_public_cities",
            "stable_fan_events",
            "stable_fan_interests",
            "stable_wallets",
        ):
            self.assertTrue(
                f"fn {helper}(" in APP,
                f"missing helper definition: {helper}",
            )
        self.assertNotIn('<For each=move || fan_events(', APP)
        self.assertNotIn('<For each=move || wallets.get()', APP)
        self.assertNotIn("collect_view()} } />", APP)

    def test_back_control_is_a_full_touch_target(self):
        start = STYLES.index("/* V9.7: larger mode-switch control")
        block = STYLES[start:]
        self.assertIn("min-width: 104px", block)
        self.assertIn("min-height: 44px", block)
        self.assertIn("z-index: 60", block)


if __name__ == "__main__":
    unittest.main()
