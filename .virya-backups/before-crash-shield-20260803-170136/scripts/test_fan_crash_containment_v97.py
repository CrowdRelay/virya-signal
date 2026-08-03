import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
APP = (ROOT / "src/app.rs").read_text(encoding="utf-8")
STYLES = (ROOT / "styles.css").read_text(encoding="utf-8")


class FanCrashContainmentV97(unittest.TestCase):
    def test_registration_uses_bounded_typed_city_collection(self):
        start = APP.index("fn FanAccess(")
        end = APP.index("fn FanApp(", start)
        block = APP[start:end]
        self.assertIn("Vec::<bridge::PublicCity>::new()", block)
        self.assertIn("open_public_city_picker(", block)
        self.assertIn("filtered_public_cities", block)
        self.assertIn("city_picker_alive", block)
        self.assertIn("bridge::load_public_cities(API_BASE)", APP)
        self.assertIn("StoredValue::new(Arc::new(AtomicBool::new(true)))", block)
        self.assertNotIn("Rc::", APP)
        self.assertIn("<For", block)
        self.assertNotIn("stable_public_cities", block)
        self.assertNotIn("refresh_public_cities", block)
        self.assertNotIn("PublicLoadingState", block)
        self.assertNotIn("bridge::pick_public_city", block)

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

    def test_remote_dashboard_lists_are_stabilized(self):
        for helper in (
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
