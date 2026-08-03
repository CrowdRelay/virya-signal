from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[1]
APP = (ROOT / "src/app.rs").read_text(encoding="utf-8")
WEB_MODELS = (ROOT / "src/models.rs").read_text(encoding="utf-8")
NATIVE = (ROOT / "src-tauri/src/lib.rs").read_text(encoding="utf-8")
NATIVE_API = (ROOT / "src-tauri/src/api.rs").read_text(encoding="utf-8")
NATIVE_MODELS = (ROOT / "src-tauri/src/models.rs").read_text(encoding="utf-8")
BRIDGE = (ROOT / "src/bridge.rs").read_text(encoding="utf-8")
STYLES = (ROOT / "styles.css").read_text(encoding="utf-8")


class SignalControlPlaneV13(unittest.TestCase):
    def test_operator_signal_is_wired_end_to_end(self):
        self.assertIn('OperatorTab::Signal', APP)
        self.assertIn('"operator_signal_overview"', APP)
        self.assertIn('operator_signal_overview,', NATIVE)
        self.assertIn('"admin/signal/overview"', NATIVE_API)
        self.assertIn('OperatorSignalOverview', WEB_MODELS)
        self.assertIn('OperatorSignalOverview', NATIVE_MODELS)

    def test_signal_is_owner_only_and_bounded(self):
        self.assertIn('owner.get()', APP)
        self.assertIn('require_owner(profile)?;', NATIVE_API)
        self.assertIn('overview.top_cities.truncate(10)', NATIVE_API)
        self.assertIn('overview.unavailable_sources.truncate(8)', NATIVE_API)
        self.assertIn('Zbiorczy obraz Sygnału bez danych osobowych fanów.', APP)

    def test_navigation_and_city_crash_boundary_stay_intact(self):
        self.assertIn('bottom-nav seven', APP)
        self.assertIn('.bottom-nav.seven', STYLES)
        self.assertNotIn('pub struct PublicCity', BRIDGE)
        self.assertNotIn('load_public_cities', BRIDGE)
        self.assertNotIn('RwSignal<Vec<bridge::PublicCity>>', APP)


if __name__ == "__main__":
    unittest.main()
