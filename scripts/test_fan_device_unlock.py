#!/usr/bin/env python3
"""Device unlock must never become the only way in, or a weaker one silently.

The fan vault is opened with 32 bytes. A PIN produces them through Argon2; a
device seal produces them at random and hands them to the platform keystore.
Both are legitimate, and the difference is invisible below the snapshot layer —
which is exactly why the rules that keep the trade honest have to be asserted
here rather than left to a reviewer to notice.
"""
from __future__ import annotations

import re
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def read(path: str) -> str:
    return (ROOT / path).read_text(encoding="utf-8")


DEVICE_UNLOCK = read("src-tauri/src/device_unlock.rs")
COMMANDS = read("src-tauri/src/commands/fan/device_unlock.rs")
SESSION = read("src-tauri/src/commands/fan/session_commerce.rs") + read(
    "src-tauri/src/commands/fan/signup.rs"
)
PLUGIN = read("src-tauri/android-push/SignalPushPlugin.kt")
GATE = read("src/app/fan.rs")
SHELL = read("src/app/fan/shell.rs")


class FanDeviceUnlockContract(unittest.TestCase):
    def test_the_password_is_never_written_in_the_clear(self) -> None:
        """Only what the keystore returned reaches the disk."""
        seal = DEVICE_UNLOCK.split("pub fn seal(", 1)[1].split("\n    pub fn ", 1)[0]
        self.assertIn("seal_device_secret", seal)
        self.assertIn("write_private_file(&sealed_path(app_data_dir), sealed.as_bytes())", seal)
        # The plaintext argument must not be what gets written.
        self.assertNotIn("write_private_file(&sealed_path(app_data_dir), password", seal)

    def test_a_vault_without_a_keystore_keeps_the_pin(self) -> None:
        """No keystore means no offer — never a password stored unsealed."""
        fallback = DEVICE_UNLOCK.split('#[cfg(not(target_os = "android"))]', 1)[1]
        self.assertIn("Err(AppError::NotConfigured)", fallback)
        self.assertNotIn("write_private_file", fallback)
        # The link path refuses a PIN-less confirmation where nothing can hold
        # the password, rather than inventing a weaker seal.
        link = SESSION.split("pub(crate) async fn fan_confirm_link(", 1)[1].split(
            "#[tauri::command]", 1
        )[0]
        self.assertIn("if !state.device_unlock_supported", link)
        self.assertIn("return Err(AppError::InvalidPin)", link)

    def test_turning_it_off_re_keys_before_the_seal_is_dropped(self) -> None:
        """A cleared key over a vault still sealed with it is an unopenable device."""
        disable = COMMANDS.split("pub(crate) async fn fan_disable_device_unlock(", 1)[1]
        self.assertIn("vault::replace_fan(&app_data_dir", disable)
        self.assertLess(
            disable.index("vault::replace_fan(&app_data_dir"),
            disable.index("crate::device_unlock::forget("),
        )

    def test_removing_the_profile_removes_the_seal(self) -> None:
        for command in ("fan_forget", "fan_delete_account"):
            body = SESSION.split(f"pub(crate) async fn {command}(", 1)[1].split(
                "#[tauri::command]", 1
            )[0]
            self.assertIn("forget_device_unlock(&state, &app).await", body, command)

    def test_the_keystore_key_is_non_exportable_and_randomised(self) -> None:
        spec = PLUGIN.split("private fun loadOrCreateDeviceKey()", 1)[1]
        self.assertIn("AndroidKeyStore", spec)
        self.assertIn("setRandomizedEncryptionRequired(true)", spec)
        self.assertIn("BLOCK_MODE_GCM", spec)
        self.assertIn("setKeySize(256)", spec)
        # A reused GCM IV breaks the mode outright, so the cipher must supply it.
        seal = PLUGIN.split("fun sealDeviceSecret(", 1)[1].split("@Command", 1)[0]
        self.assertIn("val iv = cipher.iv", seal)

    def test_a_pin_less_vault_is_not_offered_a_pin_prompt(self) -> None:
        """A PIN tried against a device-sealed vault fails like a wrong PIN."""
        self.assertIn("status.get().pin_unlock", GATE)
        self.assertIn('class:hidden=move || !status.get().pin_unlock', GATE)

    def test_the_offer_is_only_made_where_it_can_be_kept(self) -> None:
        self.assertIn("status.get().device_unlock_supported", GATE)
        self.assertIn("let seal_without_pin =", GATE)
        # The fan can always decline it.
        self.assertIn('tr("prefer_a_pin_instead")', GATE)

    def test_every_way_in_can_seal_rather_than_derive(self) -> None:
        """Signup, the mailed link and a scanned QR all reach the same choice.

        A PIN that is optional on one path and mandatory on another is a PIN
        the fan still has to invent, so the offer is worth nothing unless it
        covers every entry point.
        """
        for command in ("fan_signup", "fan_confirm_link"):
            body = SESSION.split(f"pub(crate) async fn {command}(", 1)[1].split(
                "#[tauri::command]", 1
            )[0]
            self.assertIn("pin: Option<String>", body, command)
            self.assertIn("state.device_unlock_supported", body, command)
        prepare = SESSION.split("pub(crate) async fn fan_prepare_confirmation(", 1)[1].split(
            "#[tauri::command]", 1
        )[0]
        self.assertIn("pin: None", prepare)
        scanned = SESSION.split("pub(crate) async fn fan_confirm_scanned(", 1)[1].split(
            "#[tauri::command]", 1
        )[0]
        self.assertIn("FanCredential::Device", scanned)


class MailedLinkLandsInsideTheApp(unittest.TestCase):
    """The button in the email is the fan's decision; the app must not re-ask."""

    def test_a_sealable_link_is_spent_without_a_tap(self) -> None:
        effect = GATE.split("let auto_link_attempted = RwSignal::new(false);", 1)[1].split(
            "\n    let ", 1
        )[0]
        self.assertIn("link_pending.get()", effect)
        self.assertIn("seal_without_pin()", effect)
        self.assertIn("run_confirm_without_pin()", effect)

    def test_the_link_is_spent_at_most_once(self) -> None:
        """The token is one-time; a retry would spend a credential already gone."""
        effect = GATE.split("let auto_link_attempted = RwSignal::new(false);", 1)[1].split(
            "\n    let ", 1
        )[0]
        self.assertIn("auto_link_attempted.get_untracked()", effect)
        self.assertIn("auto_link_attempted.set(true);", effect)
        self.assertLess(
            effect.index("auto_link_attempted.set(true);"),
            effect.index("run_confirm_without_pin()"),
        )
        # Submitting sets `busy`. Tracking it here would re-run this effect
        # inside its own call and cancel the request it just spawned, which is
        # exactly what happened the first time this was written.
        self.assertIn("busy.get_untracked()", effect)
        self.assertNotIn("busy.get()", effect)


class NotificationPrimer(unittest.TestCase):
    """Android 13 grants one POST_NOTIFICATIONS dialog per install.

    A denial there is permanent short of a trip to system settings, so the ask
    has to be earned by a card the fan already said yes to, and it must never
    repeat on its own.
    """

    def test_the_system_dialog_is_only_reached_through_the_card(self) -> None:
        primer = SHELL.split("fn PushPrimer(", 1)[1].split("\n#[component]", 1)[0]
        self.assertIn('"fan_push_enable"', primer)
        allow = primer.split("let allow =", 1)[1]
        self.assertIn("mark_push_primer_seen", allow)
        # Marked before the request: the system dialog takes the window away,
        # and a fan who answers it and never returns must not be asked again.
        self.assertLess(allow.index("mark_push_primer_seen"), allow.index('"fan_push_enable"'))

    def test_it_is_shown_once_and_only_when_there_is_something_to_ask(self) -> None:
        primer = SHELL.split("fn PushPrimer(", 1)[1].split("\n#[component]", 1)[0]
        self.assertIn("bridge::push_primer_seen()", primer)
        self.assertIn("status.supported", primer)
        self.assertIn("!status.enabled", primer)
        self.assertIn('status.permission != "denied"', primer)
        # Declining is remembered the same as accepting; a card that returns
        # every launch is the same nag by another name.
        dismiss = primer.split("let dismiss =", 1)[1].split("let allow =", 1)[0]
        self.assertIn("mark_push_primer_seen", dismiss)

    def test_no_storage_counts_as_already_asked(self) -> None:
        navigation = read("src/bridge/navigation.rs")
        seen = navigation.split("export function viryaPushPrimerSeen()", 1)[1].split(
            "export function", 1
        )[0]
        self.assertIn("return true;", seen)


class FanDeviceUnlockCost(unittest.TestCase):
    """The vault is the one place on this device that can take the phone down.

    Stronghold seals a snapshot with scrypt, and its default work factor wants a
    512 MiB arena per concurrent operation — enough for Android's low-memory
    killer to start evicting other apps. The PIN path already paid for that
    lesson; the device-sealed path must not reintroduce it, and the keystore
    must not add costs of its own on the startup path.
    """

    def test_the_sealed_write_and_read_go_through_the_serialised_snapshot(self) -> None:
        vault = read("src-tauri/src/vault.rs") + read("src-tauri/src/vault/device_password.rs")
        # Both helpers the device path uses are the shared ones, which take
        # `lock_snapshot()` and inherit the reduced work factor. A private
        # Stronghold call here would be a second arena nobody is holding a lock
        # for.
        sealed = vault.split("pub fn replace_fan_with_password(", 1)[1].split("\npub fn ", 1)[0]
        self.assertIn("save_bytes_with_password_at(", sealed)
        self.assertNotIn("Stronghold::default()", sealed)
        commands = COMMANDS + read("src-tauri/src/commands/fan/signup.rs")
        self.assertNotIn("Stronghold", commands)
        # The snapshot work happens on a blocking thread, never on the async
        # executor that also serves the WebView's IPC.
        unlock = COMMANDS.split("pub(crate) async fn fan_device_unlock(", 1)[1]
        self.assertIn("run_blocking(move || vault::load_fan_with_password(", unlock)

    def test_the_startup_probe_does_not_generate_a_key(self) -> None:
        """It runs on every cold start, for every install, used or not."""
        probe = PLUGIN.split("fun deviceSecretSupported(", 1)[1].split("@Command", 1)[0]
        self.assertNotIn("loadOrCreateDeviceKey", probe)
        self.assertNotIn("generateKey", probe)

    def test_the_status_path_does_not_touch_the_filesystem(self) -> None:
        """`fan_status` ends nearly every fan command; `launcher_status` is polled."""
        for source in (
            read("src-tauri/src/commands/fan/session_commerce.rs"),
            read("src-tauri/src/commands/misc.rs"),
        ):
            self.assertNotIn("device_unlock::read_mode", source)
            self.assertNotIn("device_unlock::has_sealed_password", source)
        self.assertIn("device_unlock::effective_mode(&state).await", COMMANDS)

    def test_recording_a_mode_cannot_leave_the_cache_stale(self) -> None:
        write = DEVICE_UNLOCK.split("pub async fn write_mode(", 1)[1].split("\npub async fn ", 1)[0]
        self.assertIn("*state.fan_unlock_mode.write().await = Some(mode);", write)
        clear = DEVICE_UNLOCK.split("pub async fn clear_mode(", 1)[1].split("\npub fn ", 1)[0]
        self.assertIn("*state.fan_unlock_mode.write().await = None;", clear)
        # A keystore that refuses must not keep being offered for the life of
        # the process.
        unlock = COMMANDS.split("pub(crate) async fn fan_device_unlock(", 1)[1]
        self.assertIn("invalidate_cache(&state).await", unlock)

    def test_the_plugin_holds_no_key_or_cipher_between_calls(self) -> None:
        fields = [
            line.strip()
            for line in PLUGIN.splitlines()
            if line.strip().startswith("private var") or line.strip().startswith("private val")
        ]
        for field in fields:
            for retained in ("SecretKey", "Cipher", "KeyStore", "ByteArray"):
                self.assertNotIn(retained, field, field)


if __name__ == "__main__":
    unittest.main()
