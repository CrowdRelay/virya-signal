#!/usr/bin/env python3
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

def read(path: str) -> str:
    return (ROOT / path).read_text(encoding="utf-8")

class LatarnikNativeContracts(unittest.TestCase):
    def test_beacon_identity_never_reuses_fan_identity(self):
        native = read("src-tauri/src/models/beacon.rs")
        wasm = read("src/models/beacon.rs")
        types = read("src/app/types.rs")
        self.assertIn("pub beacon_id: String", native)
        self.assertIn("pub bearer_token: String", native)
        self.assertNotIn("bearer_token", wasm)
        self.assertIn("Latarnik,", types.split("enum RootMode", 1)[1].split("}", 1)[0])
        self.assertNotIn("Latarnik", types.split("enum FanTab", 1)[1].split("}", 1)[0])

    def test_exchange_persists_vault_before_session_publication(self):
        native = read("src-tauri/src/commands/beacon.rs")
        body = native.split("async fn persist_exchanged_beacon(", 1)[1].split("#[tauri::command]", 1)[0]
        self.assertIn("client_kind: exchange.client_kind", body)
        api = read("src-tauri/src/api/beacon.rs")
        self.assertIn('"clientKind": CLIENT_KIND', api)
        self.assertIn('cfg!(target_os = "android")', api)
        self.assertIn('cfg!(target_os = "ios")', api)
        self.assertIn('} else {\n    "web"\n};', api)
        self.assertIn("vault::replace_beacon", body)
        self.assertIn("*state.beacon_session.write().await = Some(Arc::new(profile));", body)
        self.assertLess(
            body.index("vault::replace_beacon"),
            body.index("*state.beacon_session.write().await = Some(Arc::new(profile));"),
        )
        self.assertNotIn("bearer_token", read("src/app/beacon.rs"))

    def test_native_surface_covers_full_member_lifecycle(self):
        ui = read("src/app/beacon.rs")
        native = read("src-tauri/src/commands/beacon.rs")
        for command in (
            "beacon_home", "beacon_preferences_update", "beacon_press_room",
            "beacon_press_request_create", "beacon_engagement", "beacon_coverage",
            "beacon_releases", "beacon_release_confirm", "beacon_release_decline",
            "beacon_logout", "beacon_leave",
        ):
            self.assertIn(command, native)
            self.assertIn(f'"{command}"', ui)
        for tab in ("Briefing", "Radar", "Press", "Access"):
            self.assertIn(f"BeaconTab::{tab}", ui)
        self.assertIn("coverage is never", read("src/i18n/en.rs").lower())
        self.assertIn("Publikacja nie jest warunkiem", read("src/i18n/pl.rs"))

    def test_public_news_is_native_fetched_not_bundled_into_wasm(self):
        api = read("src-tauri/src/api/beacon.rs")
        ui = read("src/app/beacon.rs")
        self.assertIn("pub async fn signal_news", api)
        self.assertIn("https://virya.music/news/feed.json", api)
        self.assertIn('"beacon_news"', ui)

    def test_new_invitation_can_replace_a_revoked_configured_vault(self) -> None:
        ui = (ROOT / "src/app/beacon.rs").read_text(encoding="utf-8")
        self.assertIn("status.get().configured && !pending_link.get() && !reactivation.get()", ui)
        self.assertIn('tr("latarnik_use_new_invite")', ui)
        self.assertIn("reactivation.set(true)", ui)
        self.assertIn("reactivation.set(false)", ui)

    def test_remote_session_and_relationship_exit_actions_require_confirmation(self) -> None:
        ui = (ROOT / "src/app/beacon.rs").read_text(encoding="utf-8")
        self.assertIn("danger_action.set(Some(1))", ui)
        self.assertIn("danger_action.set(Some(2))", ui)
        self.assertIn("danger_action.set(Some(3))", ui)
        self.assertIn('tr("latarnik_logout_confirm_hint")', ui)
        self.assertIn('tr("latarnik_leave_confirm_hint")', ui)
        self.assertIn('tr("latarnik_dnc_confirm_hint")', ui)
        self.assertIn("run_danger_action(action)", ui)

    def test_access_deep_link_loads_profile_for_settings(self) -> None:
        source = (ROOT / "src/app/beacon.rs").read_text()
        access_arm = source.split("BeaconTab::Access =>", 1)[1].split("}", 1)[0]
        self.assertIn("refresh_beacon_home(home, loading_home, error)", access_arm)
        self.assertIn("refresh_beacon_requests(requests, error)", access_arm)
        self.assertIn("refresh_beacon_releases(releases, error)", access_arm)

    def test_destructive_confirmation_reuses_panel_styling(self) -> None:
        source = (ROOT / "src/app/beacon.rs").read_text()
        self.assertIn('class="beacon-delivery-form beacon-danger-confirm"', source)

    def test_release_delivery_does_not_label_required_recipient_name_optional(self) -> None:
        ui = (ROOT / "src/app/beacon.rs").read_text(encoding="utf-8")
        access = ui.split("fn BeaconAccessHub", 1)[1].split("fn BeaconSettings", 1)[0]
        self.assertIn('tr("latarnik_recipient_name")', access)
        self.assertNotIn('tr("name_optional")', access)

    def test_physical_release_decline_requires_a_second_explicit_action(self) -> None:
        ui = (ROOT / "src/app/beacon.rs").read_text(encoding="utf-8")
        self.assertIn("decline_candidate.set(Some", ui)
        self.assertIn("confirm_decline_release", ui)
        self.assertIn('tr("latarnik_decline_release_confirm_title")', ui)
        self.assertIn("decline_candidate.set(None)", ui)

    def test_leptos_handlers_do_not_reintroduce_known_fn_once_or_clippy_traps(self) -> None:
        ui = read("src/app/beacon.rs")
        self.assertNotIn("*v=!*v", ui)
        self.assertIn("*v = !*v", ui)
        self.assertNotIn("help_submit_id", ui)
        self.assertIn("if let Some(id) = helping.get_untracked()", ui)
        self.assertNotIn("<Show when=move || can_respond>", ui)
        self.assertIn("let actions = can_respond.then(||", ui)
        self.assertIn("let confirm_id = campaign_id.clone();", ui)


    def test_wasm_beacon_dto_is_projection_only_and_resume_loop_is_idiomatic(self) -> None:
        models = read("src/models/beacon.rs")
        app = read("src/app.rs")
        # The WASM side only deserializes fields it renders. Native models retain
        # the full API contract; serde safely ignores additional map fields here.
        for unused in (
            "pub beacon_id: String", "pub beacon_kind: String", "pub expires_at: String",
            "pub slug: String", "pub ticket_url: Option<String>", "pub help_kind: Option<String>",
            "pub open_press_requests: i64", "pub published_at: String", "pub image_url: String",
        ):
            self.assertNotIn(unused, models)
        self.assertIn("pub display_name: String", models)
        self.assertIn("pub nearby_events: Vec<BeaconNearbyEvent>", models)
        self.assertIn("pub assets: Vec<BeaconPressAsset>", models)
        self.assertIn("pub struct BeaconEngagementResult {}", models)
        self.assertIn("pub struct BeaconMutationResult {}", models)
        self.assertIn("for (registered, event) in events.iter().enumerate()", app)
        self.assertNotIn("let mut registered = 0_usize", app)

    def test_pending_app_link_survives_a_cancelled_scan_and_fires_once(self) -> None:
        ffi = read("src/bridge/ffi.rs")
        app = read("src/app.rs")
        native = read("src-tauri/src/commands/beacon.rs")
        scan = ffi.split("export async function viryaScanAndConfirmBeacon()", 1)[1].split("\n}\n", 1)[0]
        # An abandoned camera pass drops the staged PIN only; the App Link
        # capability is a separate invitation and must survive it.
        self.assertIn("beacon_clear_pending_confirmation", scan)
        self.assertNotIn("beacon_clear_pending_invite", scan)
        confirmation = native.split("async fn beacon_clear_pending_confirmation", 1)[1].split("#[tauri::command]", 1)[0]
        self.assertNotIn("pending_beacon_link", confirmation)
        self.assertIn("pending_beacon_link", native.split("async fn beacon_clear_pending_invite", 1)[1])
        # The native side keeps reporting a queued link, so only the transition
        # may relock the session and force the mode.
        self.assertIn("Ok(true) if !beacon_pending_link.get_untracked() =>", app)

    def test_press_room_is_fetched_once_per_entry(self) -> None:
        ui = read("src/app/beacon.rs")
        tab_effect = ui.split("BeaconTab::Radar =>", 1)[1].split("BeaconTab::Access =>", 1)[0]
        # invoke_latest is latest-wins dedup, not a cache, so a second call here
        # would be a second real round trip with its answer discarded.
        self.assertNotIn("refresh_beacon_press_room", tab_effect)
        press = ui.split("fn BeaconPressRoom", 1)[1].split("fn BeaconAccessHub", 1)[0]
        self.assertEqual(press.count("refresh_beacon_press_room"), 1)
        self.assertIn("refresh.get();", press)

    def test_accreditation_is_always_event_scoped(self) -> None:
        ui = (ROOT / "src/app/beacon.rs").read_text(encoding="utf-8")
        self.assertIn('let request_kind = RwSignal::new("press_photo".to_owned())', ui)
        self.assertIn('event_selected && current_kind == "press_photo"', ui)
        self.assertIn('request_kind.get()=="accreditation" && selected_event.get().is_none()', ui)
        self.assertIn('<Show when=move || selected_event.get().is_some()><option value="accreditation">', ui)

if __name__ == "__main__":
    unittest.main()
