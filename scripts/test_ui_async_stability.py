from __future__ import annotations

import re
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def function_body(source: str, name: str) -> str:
    match = re.search(rf"(?m)^fn {re.escape(name)}\(", source)
    if match is None:
        raise AssertionError(f"missing function: {name}")
    start = match.start()
    next_match = re.search(r"(?m)^fn [A-Za-z0-9_]+\(", source[match.end() :])
    end = match.end() + next_match.start() if next_match else len(source)
    return source[start:end]


class UiAsyncStabilityContracts(unittest.TestCase):
    def test_stale_latest_requests_do_not_clear_newer_loading_state(self) -> None:
        support = (ROOT / "src/app/support.rs").read_text(encoding="utf-8")
        guarded = (
            "refresh_operator_events",
            "refresh_operator_qr",
            "refresh_operator_signal",
            "refresh_operator_autopilot",
            "refresh_operator_chief",
            "refresh_operator_ops",
            "refresh_fan_home",
            "refresh_fan_events",
            "refresh_fan_merch",
            "refresh_fan_referral",
            "refresh_fan_interests",
            "refresh_fan_admission_pass",
            "refresh_fan_area",
            "refresh_wallets",
        )
        self.assertIn("!matches!(result, Ok(None))", support)
        for name in guarded:
            body = function_body(support, name)
            self.assertIn("invoke_latest", body, name)
            self.assertIn("latest_request_completed(&result)", body, name)
            self.assertIn("if completed", body, name)
            self.assertLess(
                body.index("latest_request_completed(&result)"),
                body.rindex("if completed"),
                name,
            )

        checkout = (ROOT / "src/app/fan/events.rs").read_text(encoding="utf-8")
        checkout = checkout.split("fn FanTicketCheckout", 1)[1]
        self.assertIn("latest_request_completed(&result)", checkout)
        self.assertIn("if completed {\n                sale_loading.set(false);", checkout)


    def test_launcher_resume_refresh_is_latest_wins(self) -> None:
        app = (ROOT / "src/app.rs").read_text(encoding="utf-8")
        client = (ROOT / "src/bridge/client.rs").read_text(encoding="utf-8")
        self.assertIn('"launcher:status"', client)
        self.assertIn("Result<Option<crate::models::LauncherStatus>, String>", client)
        self.assertIn("let completed = latest_request_completed(&result);", app)
        self.assertIn("Ok(None) => {}", app)
        self.assertIn("if completed {", app)

    def test_push_resume_waits_for_in_flight_settings_command(self) -> None:
        source = (ROOT / "src/app/fan_home.rs").read_text(encoding="utf-8")
        push = source.split("fn NativePushControl", 1)[1]
        self.assertIn("resume_refresh.get();", push)
        self.assertIn("busy.get()", push)
        self.assertNotIn("busy.get_untracked() {\n            return;", push.split("let toggle", 1)[0])
        self.assertIn("enable_after_settings", push)

    def test_fan_refresh_is_explicit_instead_of_a_hidden_double_click(self) -> None:
        shell = (ROOT / "src/app/fan/shell.rs").read_text(encoding="utf-8")
        self.assertIn('on:click=refresh_all', shell)
        self.assertIn('tr("refresh_all_data")', shell)
        self.assertNotIn('on:dblclick=', shell)

    def test_operator_refresh_is_explicit_and_cleanup_invalidates_reads(self) -> None:
        shell = (ROOT / "src/app/operator/shell.rs").read_text(encoding="utf-8")
        self.assertIn('on:click=refresh_all', shell)
        self.assertIn('tr("refresh_all_data")', shell)
        self.assertIn('on_cleanup(move || bridge::invalidate_latest("operator:"))', shell)
        self.assertNotIn('on:dblclick=', shell)

    def test_notification_launch_target_survives_cold_and_warm_app_paths(self) -> None:
        app = (ROOT / "src/app.rs").read_text(encoding="utf-8")
        fan = (ROOT / "src/app/fan.rs").read_text(encoding="utf-8")
        shell = (ROOT / "src/app/fan/shell.rs").read_text(encoding="utf-8")
        plugin = (ROOT / "src-tauri/android-push/SignalPushPlugin.kt").read_text(encoding="utf-8")
        service = (ROOT / "src-tauri/android-push/ViryaFirebaseMessagingService.kt").read_text(encoding="utf-8")
        self.assertIn('"fan_push_take_target"', app)
        self.assertIn("push_target=push_target", app)
        self.assertIn("push_target=push_target", fan)
        self.assertIn("fan_tab_for_push_target", shell)
        self.assertIn('target.contains("event=")', shell)
        self.assertIn("override fun onNewIntent", plugin)
        self.assertIn("takeLaunchTarget", plugin)
        self.assertIn('putExtra("virya_push_target_path", targetPath)', service)

    def test_toast_timeout_cannot_clear_a_newer_message(self) -> None:
        support = (ROOT / "src/app/support.rs").read_text(encoding="utf-8")
        toast = support.split("fn Toast", 1)[1].split("fn latest_request_completed", 1)[0]
        self.assertIn("dismiss_generation", toast)
        self.assertIn("get_untracked().wrapping_add(1)", toast)
        self.assertIn("dismiss_generation.try_get_untracked() == Some(generation)", toast)
        self.assertIn("error.try_set(None)", toast)
        self.assertNotIn("set_timeout(move || error.set(None)", toast)


if __name__ == "__main__":
    unittest.main()
