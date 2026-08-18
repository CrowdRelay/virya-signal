from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[1]


def read(path: str) -> str:
    return (ROOT / path).read_text(encoding="utf-8")


class PrincipalStaffHardeningTests(unittest.TestCase):
    def test_retired_boot_and_launcher_sources_stay_deleted(self):
        self.assertFalse((ROOT / "boot-initializer.mjs").exists())
        self.assertFalse((ROOT / "src-tauri" / "launcher-assets").exists())
        ignored = read(".gitignore")
        self.assertIn("/boot-initializer.mjs", ignored)
        self.assertIn("/src-tauri/launcher-assets/", ignored)

    def test_mobile_smoke_tracks_root_boot_dependencies(self):
        workflow = read(".github/workflows/mobile-smoke.yml")
        for path in ("boot-i18n.js", "runtime-i18n.js", "bundle-stage-pack.webp"):
            self.assertGreaterEqual(workflow.count(path), 2, path)

    def test_firebase_runtime_can_self_initialize_from_compiled_resources(self):
        kotlin = read("src-tauri/android-push/SignalPushPlugin.kt")
        rust = read("src-tauri/src/push_plugin.rs")
        fan = read("src-tauri/src/commands/fan/push.rs")
        self.assertIn("FirebaseApp.initializeApp(context)", kotlin)
        self.assertIn("ensureFirebaseInitialized", kotlin)
        self.assertIn("getFirebaseState", kotlin)
        self.assertIn('"getFirebaseState"', rust)
        self.assertIn("native_firebase_configured", fan)
        self.assertIn('"firebase_not_configured"', fan)

    def test_fcm_provider_token_is_device_scoped_not_audience_scoped(self):
        rust_push = read("src-tauri/src/push_plugin.rs")
        kotlin_plugin = read("src-tauri/android-push/SignalPushPlugin.kt")
        fan_push = read("src-tauri/src/commands/fan/push.rs")
        fan_session = read("src-tauri/src/commands/fan/session_commerce.rs")
        for source in (rust_push, kotlin_plugin, fan_push, fan_session):
            self.assertNotIn("deleteToken", source)
            self.assertNotIn("delete_native_push_token", source)
        self.assertIn("fan_disable_android_push", fan_push)
        self.assertIn("fan_disable_android_push", fan_session)
        self.assertIn("fan_push_disable_not_confirmed", fan_session)
        self.assertIn("fan_push_disable_not_confirmed", fan_push)
        self.assertIn("ok_or(AppError::Locked)?", fan_session)
        self.assertNotIn("remote disable before forget degraded", fan_session)

    def test_staff_push_disable_is_durable_and_serialized(self):
        lib = read("src-tauri/src/lib.rs")
        commands = read("src-tauri/src/commands/operator.rs")
        api = read("src-tauri/src/api/operator.rs")
        ui = read("src/app/operator/checklist.rs")
        self.assertIn("operator_push_mutation: Mutex<()>", lib)
        self.assertIn("operator_push_mutation: Mutex::new(())", lib)
        self.assertGreaterEqual(commands.count("state.operator_push_mutation.lock().await"), 4)
        self.assertIn("staff-push-preference-v1.json", commands)
        self.assertIn("operator_disable_android_push", commands)
        self.assertIn('"staff/push/endpoints/disable"', api)
        self.assertIn('"operator_push_disable"', ui)
        self.assertIn("operator_push_disable", lib)

    def test_forget_is_fail_closed_for_authenticated_staff_push_cleanup(self):
        commands = read("src-tauri/src/commands/operator.rs")
        start = commands.index("pub(crate) async fn forget_device")
        end = commands.index("pub(crate) async fn operator_events", start)
        body = commands[start:end]
        self.assertIn("operator_disable_android_push", body)
        self.assertIn(".await?", body)
        self.assertIn("staff_push_disable_not_confirmed", body)
        self.assertIn("ok_or(AppError::Locked)?", body)
        self.assertLess(body.index("operator_disable_android_push"), body.index("vault::remove"))
        self.assertNotIn("remote disable before forget degraded", body)

    def test_owner_only_control_plane_methods_require_owner(self):
        api = read("src-tauri/src/api/operator.rs")
        owner_only = (
            "issue_pass",
            "revoke_pass",
            "operator_signal_overview",
            "operator_ops_overview",
            "operator_autopilot_overview",
            "operator_autopilot_chief_of_staff",
            "operator_autopilot_set_authority",
            "operator_autopilot_assign",
            "operator_autopilot_approve",
            "operator_autopilot_cancel",
            "operator_retry",
        )
        for index, name in enumerate(owner_only):
            start = api.index(f"pub async fn {name}")
            next_starts = [api.find("pub async fn ", start + 1)]
            end = min((value for value in next_starts if value != -1), default=len(api))
            self.assertIn("require_owner(profile)?", api[start:end], name)

    def test_staff_pairing_expiry_is_preserved_and_visible(self):
        native_models = read("src-tauri/src/models/session_fan.rs")
        pairing = read("src-tauri/src/commands/pairing.rs")
        frontend_models = read("src/models.rs")
        settings = read("src/app/operator/commerce_settings.rs")
        self.assertIn("session_expires_at: Option<u64>", native_models)
        self.assertIn("session_expires_at: Some(exchange.expires_at)", pairing)
        self.assertIn("session_expires_at: Option<u64>", frontend_models)
        self.assertIn("staff_session_expired_pair_again", settings)
        self.assertIn("staff_session_expires_soon_pair_again", settings)


    def test_pairing_cannot_mint_owner_profile(self):
        pairing = read("src-tauri/src/commands/pairing.rs")
        self.assertIn("pairing.role != OperatorRole::Staff", pairing)
        self.assertIn("exchange.role != OperatorRole::Staff", pairing)
        self.assertNotIn("OperatorRole::Owner {", pairing)


if __name__ == "__main__":
    unittest.main()
