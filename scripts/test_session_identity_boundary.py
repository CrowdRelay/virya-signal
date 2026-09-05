#!/usr/bin/env python3
"""Contracts for the identity boundary: adopting a session, destroying one, and
never reporting a local queue as a server outcome.

Three invariants live here because all three were violated by code that read
correctly line by line:

1. Establishing a session retires the `launcher:` request scope. `launcher_status`
   is one command with one constant argument, so a read still on the wire from a
   resume tick is coalesced with the read issued right after a successful unlock —
   and answers it with the pre-unlock snapshot, putting the user straight back on
   the lock screen.

2. Destroying an identity retires the capabilities staged for it. Locking is
   temporary and deliberately keeps them; forgetting, deleting, leaving and
   logging out are not, and a one-time credential that outlives its account is
   offered to whoever sets the device up next.

3. A submission parked in the on-disk outbox is not a submission CrowdRelay
   accepted, and the app must not say it is.
"""

from __future__ import annotations

import re
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
LAUNCHER_INVALIDATION = 'bridge::invalidate_latest("launcher:")'


def read(relative: str) -> str:
    return (ROOT / relative).read_text(encoding="utf-8")


def item_body(source: str, signature: str) -> str:
    """The source of one `fn`/`let` item, up to the next item at column zero
    or, for a `let` binding, up to the closing `};` of its block."""
    start = source.find(signature)
    if start == -1:
        raise AssertionError(f"missing item: {signature}")
    if signature.startswith("let "):
        end = source.index("\n    };", start)
        return source[start:end]
    next_item = re.search(r"(?m)^(?:pub(?:\(crate\))? )?(?:async )?fn ", source[start + len(signature):])
    end = start + len(signature) + next_item.start() if next_item else len(source)
    return source[start:end]


class LauncherScopeRetirement(unittest.TestCase):
    """Every path that publishes a freshly established native session first
    retires the scope the launcher read shares with the pre-session one."""

    def test_the_launcher_scope_has_exactly_one_reader(self) -> None:
        client = read("src/bridge/client.rs")
        self.assertIn('"launcher:status"', client)
        self.assertEqual(client.count('"launcher:status"'), 1)

    def test_fan_adoption_funnel_retires_the_scope(self) -> None:
        fan = read("src/app/fan.rs")
        body = item_body(fan, "fn adopt_fan_session(")
        self.assertIn(LAUNCHER_INVALIDATION, body)
        # Confirmation, mailed link and scanned QR all publish through the
        # funnel rather than writing the status signal themselves.
        for caller in ("fn run_fan_confirmation(", "fn submit_fan_confirm_link("):
            self.assertIn("adopt_fan_session(", item_body(fan, caller), caller)
        # The two paths that do not go through the funnel carry their own.
        for command in ('"fan_unlock"', '"fan_signup"', '"fan_device_unlock"'):
            block = fan.split(command, 1)[1][:2400]
            self.assertIn(LAUNCHER_INVALIDATION, block, command)

    def test_staff_adoption_funnel_retires_the_scope(self) -> None:
        operator = read("src/app/operator.rs")
        self.assertIn(LAUNCHER_INVALIDATION, item_body(operator, "fn adopt_operator_session("))
        # Unlock, pairing and manual configuration are the three ways a staff
        # session is established, and none of them may publish it directly.
        self.assertEqual(operator.count("adopt_operator_session(status, value)"), 3)
        self.assertNotIn("\n                    status.set(value);", operator)

    def test_beacon_adoption_retires_the_scope(self) -> None:
        beacon = read("src/app/beacon.rs")
        adopt = item_body(beacon, "let adopt = move |value: BeaconSessionStatus| {")
        self.assertIn(LAUNCHER_INVALIDATION, adopt)
        # The scanner path publishes without the closure, so it carries its own.
        scan = beacon.split("bridge::scan_and_confirm_beacon()", 1)[1][:1200]
        self.assertIn(LAUNCHER_INVALIDATION, scan)


class StagedCapabilityLifetime(unittest.TestCase):
    """A one-time capability may outlive a lock. It may not outlive the identity."""

    def test_beacon_leaving_and_logging_out_drop_queued_invitations(self) -> None:
        beacon = read("src-tauri/src/commands/beacon.rs")
        cleared = item_body(beacon, "async fn clear_beacon_session_state(")
        for field in (
            "beacon_session",
            "beacon_pin",
            "beacon_vault_password",
            "pending_beacon_confirmation",
            "pending_beacon_link",
        ):
            self.assertIn(field, cleared, field)
        for command in ("async fn beacon_logout(", "async fn beacon_leave("):
            self.assertIn("clear_beacon_session_state(&state)", item_body(beacon, command), command)
        # Locking is temporary: the invitation is still the member's to spend.
        self.assertNotIn("pending_beacon_link", item_body(beacon, "async fn beacon_lock("))

    def test_fan_forgetting_and_deleting_drop_staged_fan_credentials(self) -> None:
        session = read("src-tauri/src/commands/fan/session_commerce.rs")
        cleared = item_body(session, "async fn clear_fan_identity_capabilities(")
        self.assertIn("pending_fan_confirm_token", cleared)
        self.assertIn("pending_synesthesia_handoff", cleared)
        for command in ("async fn fan_forget(", "async fn fan_delete_account("):
            self.assertIn(
                "clear_fan_identity_capabilities(&state)", item_body(session, command), command
            )
        # Same reasoning as the Beacon lock: a link that arrived while the vault
        # was closed is still the same fan's once they open it again.
        lock = item_body(session, "async fn fan_lock(")
        self.assertNotIn("pending_synesthesia_handoff", lock)
        self.assertNotIn("pending_fan_confirm_token", lock)


class DomainSeparation(unittest.TestCase):
    """Fan, operator and Beacon are three security domains sharing one process.

    Nothing enforces that structurally today beyond which accessor a command
    happens to call: `AppState` holds all three sets of credentials and every
    command can reach every field. That is fine while each module only touches
    its own, and this is the check that says so out loud — so a future command
    that reads a fan bearer from the operator surface fails here instead of
    shipping.
    """

    DOMAIN_STATE = {
        "fan": ("fan_profile", "fan_pin", "fan_vault_password", "wallet_qr_tokens"),
        "operator": ("operator_profile", "operator_pin", "operator_vault_password"),
        "beacon": ("beacon_profile", "beacon_pin", "beacon_vault_password"),
    }
    MODULE_DOMAIN = {
        "commands/fan.rs": "fan",
        "commands/fan/device_unlock.rs": "fan",
        "commands/fan/push.rs": "fan",
        "commands/fan/session_commerce.rs": "fan",
        "commands/fan/signup.rs": "fan",
        "commands/fan/wallet.rs": "fan",
        "commands/synesthesia.rs": "fan",
        "commands/operator.rs": "operator",
        "commands/show_mode.rs": "operator",
        "commands/beacon.rs": "beacon",
        "commands/beacon/push.rs": "beacon",
    }

    def test_no_command_module_reaches_into_another_domain(self) -> None:
        for module, own in self.MODULE_DOMAIN.items():
            source = read(f"src-tauri/src/{module}")
            for domain, fields in self.DOMAIN_STATE.items():
                if domain == own:
                    continue
                for field in fields:
                    self.assertNotIn(
                        field,
                        source,
                        f"{module} belongs to the {own} domain and must not touch {field}",
                    )

    def test_the_launcher_summary_reads_presence_only(self) -> None:
        # `launcher_status` is the one command that spans all three domains. It
        # may report whether each is configured/unlocked; it may not read a
        # credential to do it.
        misc = read("src-tauri/src/commands/misc.rs")
        for fields in self.DOMAIN_STATE.values():
            for field in fields:
                self.assertNotIn(field, misc, field)


class OfflineWriteHonesty(unittest.TestCase):
    def test_queued_feedback_is_never_reported_as_delivered(self) -> None:
        native = read("src-tauri/src/commands/misc.rs")
        body = item_body(native, "pub(crate) async fn submit_anonymous_feedback(")
        self.assertIn("Result<String, AppError>", body)
        self.assertIn("Ok(FEEDBACK_SENT.to_owned())", body)
        self.assertIn("Ok(FEEDBACK_QUEUED.to_owned())", body)
        # The queued outcome is produced only by the enqueue branch.
        self.assertLess(body.index("feedback_queue::enqueue"), body.index("FEEDBACK_QUEUED"))

        ui = read("src/app/fan/wallet.rs")
        submit = ui.split('"submit_anonymous_feedback"', 1)[1][:1200]
        self.assertIn('outcome == "queued"', submit)
        self.assertIn("feedback_queued_until_online", submit)
        self.assertIn("feedback_was_sent_anonymously_thank_you", submit)

        for language in ("en", "pl"):
            catalog = read(f"src/i18n/{language}.rs")
            self.assertIn('"feedback_queued_until_online" =>', catalog, language)


if __name__ == "__main__":
    unittest.main()
