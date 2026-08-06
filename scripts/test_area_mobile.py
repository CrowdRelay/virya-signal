import re
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]


class MobileAreaContracts(unittest.TestCase):
    def test_native_geolocation_is_mobile_only_and_least_privilege(self):
        cargo = (ROOT / "src-tauri/Cargo.toml").read_text()
        native = (ROOT / "src-tauri/src/lib.rs").read_text()
        capability = (ROOT / "src-tauri/capabilities/mobile.json").read_text()
        self.assertIn('tauri-plugin-geolocation = "2.3.2"', cargo)
        self.assertIn("#[cfg(mobile)]", native)
        self.assertIn("tauri_plugin_geolocation::init()", native)
        self.assertIn('"geolocation:allow-check-permissions"', capability)
        self.assertIn('"geolocation:allow-request-permissions"', capability)
        self.assertIn('"geolocation:allow-get-current-position"', capability)
        self.assertNotIn('"geolocation:allow-watch-position"', capability)

    def test_area_claim_stays_native_and_bounded(self):
        bridge = (ROOT / "src/bridge.rs").read_text()
        commands = (ROOT / "src-tauri/src/commands/fan.rs").read_text()
        api = (ROOT / "src-tauri/src/api/public.rs").read_text()
        self.assertIn("plugin:geolocation|get_current_position", bridge)
        self.assertIn("viryaCollectLocationSamples", bridge)
        self.assertIn("samples.len() < 3", commands)
        self.assertIn("samples.len() > 8", commands)
        self.assertIn('const AREA_CHALLENGE_URL: &str = "https://virya.music/api/area/challenge"', api)
        self.assertIn('const AREA_CLAIM_URL: &str = "https://virya.music/api/area/claim"', api)
        self.assertIn('.bearer_auth(profile.fan_session_token.trim())', api)
        self.assertIn('.header("x-virya-area-wallet", wallet_id.to_string())', api)

    def test_app_uses_only_coarse_public_city_coordinates(self):
        source = (ROOT / "src/app/area.rs").read_text()
        coordinates = re.findall(r"approximate_(?:lat|lng):\s*(-?\d+(?:\.(\d+))?)", source)
        self.assertGreater(len(coordinates), 0)
        for _, decimals in coordinates:
            self.assertLessEqual(len(decimals or ""), 1)
        self.assertNotIn("radius_meters", source)
        self.assertIn("area_location_privacy", source)
        self.assertIn("fan_area_challenge", source)
        self.assertIn("fan_area_claim", source)

    def test_area_ui_keeps_inactive_cities_visible(self):
        source = (ROOT / "src/app/area.rs").read_text()
        self.assertIn("AREA_DROPS.to_vec()", source)
        self.assertIn("class:is-live", source)
        self.assertIn("inactive_area_point", source)
        self.assertNotIn("hidden=move || !live", source)


    def test_touch_targets_and_area_layout_use_shared_controls(self):
        styles = (ROOT / "styles.css").read_text()
        self.assertEqual(styles.count(".back-button {"), 1)
        self.assertRegex(styles, r"\.back-button \{[^}]*min-height: 44px")
        self.assertRegex(styles, r"\.area-native-marker \{[^}]*width: 44px; height: 44px")
        self.assertIn(
            ".area-target-actions,\n  .area-native-actions { grid-template-columns: 1fr; }",
            styles,
        )
        self.assertIn(".ticket-pool-status", styles)

    def test_area_api_error_codes_are_localized(self):
        public_api = (ROOT / "src-tauri/src/api/public.rs").read_text()
        http = (ROOT / "src-tauri/src/api/http.rs").read_text()
        self.assertIn("decode_with_error_mapper", http)
        for code in [
            "DROP_INACTIVE",
            "CHALLENGE_INVALID",
            "RATE_LIMITED",
            "NOT_ENOUGH_SAMPLES",
            "LOW_ACCURACY",
            "OUTSIDE_ZONE",
            "DROP_FULL",
            "TEMPORARY",
        ]:
            self.assertIn(f'"{code}"', public_api)

if __name__ == "__main__":
    unittest.main()
