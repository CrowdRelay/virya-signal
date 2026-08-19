#!/usr/bin/env python3
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
APP = (ROOT / "src/app.rs").read_text()
FAN = (ROOT / "src/app/fan.rs").read_text()
SHELL = (ROOT / "src/app/fan/shell.rs").read_text()
BEACON = "\n".join(
    [(ROOT / "src/app/beacon.rs").read_text()]
    + [p.read_text() for p in sorted((ROOT / "src/app/beacon").glob("*.rs"))]
)
TYPES = (ROOT / "src/app/types.rs").read_text()
PL = (ROOT / "src/i18n/pl.rs").read_text()
EN = (ROOT / "src/i18n/en.rs").read_text()

class LatarnikSignalEntryContract(unittest.TestCase):
    def test_latarnik_is_a_first_class_mode_not_a_fan_tab(self):
        self.assertIn("Latarnik,", TYPES.split("enum RootMode", 1)[1])
        fan_tab = TYPES.split("enum FanTab", 1)[1].split("}", 1)[0]
        self.assertNotIn("Latarnik", fan_tab)
        self.assertIn("RootMode::Latarnik =>", APP)
        self.assertIn("<BeaconPortal", APP)
        self.assertIn("mode.set(RootMode::Latarnik)", FAN)
        self.assertIn("mode.set(RootMode::Latarnik)", SHELL)

    def test_warm_app_link_forces_old_beacon_session_back_to_access(self):
        app = (ROOT / "src/app.rs").read_text(encoding="utf-8")
        app_link = app.split('"beacon_take_app_link"', 1)[1].split("Effect::new", 1)[0]
        self.assertIn('"beacon_lock"', app_link)
        self.assertIn("beacon_status.set(value)", app_link)
        self.assertLess(app_link.index('"beacon_lock"'), app_link.index("beacon_pending_link.set(true)"))
        self.assertLess(app_link.index("beacon_pending_link.set(true)"), app_link.index("mode.set(RootMode::Latarnik)"))

    def test_pending_app_link_wins_stale_launcher_snapshot_and_can_be_cancelled(self):
        beacon = (ROOT / "src/app/beacon.rs").read_text(encoding="utf-8")
        native = (ROOT / "src-tauri/src/commands/beacon.rs").read_text(encoding="utf-8")
        portal = beacon.split("fn BeaconPortal", 1)[1].split("fn BeaconAccess", 1)[0]
        access = beacon.split("fn BeaconAccess", 1)[1].split("fn BeaconApp", 1)[0]
        clear = native.split("fn beacon_clear_pending_invite", 1)[1].split("#[tauri::command]", 1)[0]
        self.assertIn("status.get().unlocked && !pending_link.get()", portal)
        self.assertIn('"beacon_clear_pending_invite"', access)
        self.assertIn('"beacon_status"', access)
        self.assertIn("pending_beacon_confirmation", clear)
        self.assertIn("pending_beacon_link", clear)

    def test_web_portal_remains_an_explicit_fallback(self):
        self.assertIn("https://virya.music/pl/latarnik/", BEACON)
        self.assertIn("https://virya.music/latarnik/", BEACON)
        self.assertIn("open_external_url", BEACON)
        # Fan surfaces no longer leak the invite into an external browser by default.
        self.assertNotIn("https://virya.music/pl/latarnik/", FAN)
        self.assertNotIn("https://virya.music/pl/latarnik/", SHELL)

    def test_copy_exists_in_both_languages(self):
        for key in ("latarnik_zone", "latarnik_short_pitch", "latarnik_native_label", "latarnik_not_street_team"):
            self.assertIn(f'"{key}"', PL)
            self.assertIn(f'"{key}"', EN)
    def test_last_member_surface_is_remembered_without_persisting_staff_mode(self) -> None:
        app = (ROOT / "src/app.rs").read_text(encoding="utf-8")
        # Root-mode/tab storage moved out of the monolithic ffi module into
        # bridge/navigation.rs; the guarantees below are about the bridge surface.
        ffi = "\n".join(
            (ROOT / name).read_text(encoding="utf-8")
            for name in ("src/bridge/ffi.rs", "src/bridge/navigation.rs")
        )
        self.assertIn("persisted_root_mode()", app)
        self.assertIn('RootMode::Latarnik => bridge::set_root_mode_state("latarnik")', app)
        # Team is now restored across the i18n reload, so the old combined arm
        # no longer exists. The guarantee it protected is unchanged and is
        # asserted directly: the staff gate is never persisted, team lives in
        # sessionStorage under its own key so it dies with the WebView session,
        # and the durable localStorage read is clamped to fan/latarnik so a
        # tampered value can never boot straight into the operator surface.
        self.assertIn('RootMode::StaffGate => {}', app)
        self.assertIn("VIRYA_ROOT_MODE_STORAGE_KEY", ffi)
        self.assertIn("window.localStorage?.setItem(VIRYA_ROOT_MODE_STORAGE_KEY, safe)", ffi)
        self.assertIn(
            "window.sessionStorage?.setItem(VIRYA_TRANSIENT_ROOT_MODE_STORAGE_KEY, 'team')", ffi
        )
        self.assertIn("return value === 'latarnik' ? 'latarnik' : 'fan'", ffi)
        self.assertNotIn(
            "localStorage?.setItem(VIRYA_ROOT_MODE_STORAGE_KEY, 'team')", ffi
        )


if __name__ == "__main__":
    unittest.main()
