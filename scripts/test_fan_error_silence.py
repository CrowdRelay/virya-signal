#!/usr/bin/env python3
"""Fan background refresh functions must not surface transient errors.

The fan never sees a toast for network/timeout/connection failures — the
app recovers silently via cached data + the next refresh cycle. User-action
errors (checkout, import) use inline status, not the shared error signal.
Staff/operator refresh functions are exempt: staff needs errors for debugging.
"""
from __future__ import annotations

import re
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SUPPORT = (ROOT / "src/app/support.rs").read_text(encoding="utf-8")
FAN_HOME = (ROOT / "src/app/fan_home.rs").read_text(encoding="utf-8")
WALLET = (ROOT / "src/app/fan/wallet.rs").read_text(encoding="utf-8")
EVENTS = (ROOT / "src/app/fan/events.rs").read_text(encoding="utf-8")
SHELL = (ROOT / "src/app/fan/shell.rs").read_text(encoding="utf-8")


def extract_fn(source: str, name: str) -> str:
    """Extract a function body from Rust source by name."""
    pattern = rf"fn {re.escape(name)}\b"
    match = re.search(pattern, source)
    if not match:
        return ""
    start = match.start()
    rest = source[start:]
    # Find the end by brace counting from the first opening brace
    brace_start = rest.index("{")
    depth = 0
    for i, ch in enumerate(rest[brace_start:], brace_start):
        if ch == "{":
            depth += 1
        elif ch == "}":
            depth -= 1
            if depth == 0:
                return rest[: i + 1]
    return rest


class FanErrorSilenceTests(unittest.TestCase):
    def test_fan_refresh_functions_do_not_set_error_on_err(self) -> None:
        """All fan_* refresh functions must swallow Err silently."""
        fan_fns = [
            "refresh_fan_home",
            "refresh_fan_events",
            "refresh_fan_merch",
            "refresh_fan_merch_bundles",
            "refresh_fan_referral",
            "refresh_fan_interests",
            "refresh_fan_admission_pass",
            "refresh_fan_area",
            "refresh_wallets",
        ]
        for fn_name in fan_fns:
            body = extract_fn(SUPPORT, fn_name)
            self.assertTrue(body, f"could not find function {fn_name}")
            # Must not contain set_error_debounced (used by operator functions)
            self.assertNotIn(
                "set_error_debounced(error,",
                body,
                f"{fn_name} still surfaces errors via set_error_debounced",
            )
            # Must not set raw error messages from Err branches
            self.assertNotIn(
                "error.set(Some(message))",
                body,
                f"{fn_name} still surfaces raw error messages",
            )
            # Must not set debounced error from Err branches
            self.assertNotIn(
                "set_error_debounced(error, message)",
                body,
                f"{fn_name} still surfaces debounced errors",
            )

    def test_fan_merch_does_not_clear_on_error(self) -> None:
        """refresh_fan_merch must keep the last catalog on error, not clear it."""
        body = extract_fn(SUPPORT, "refresh_fan_merch")
        self.assertNotIn("merch.set(None)", body)

    def test_fan_merch_bundles_does_not_clear_on_error(self) -> None:
        """refresh_fan_merch_bundles must keep last bundles on error."""
        body = extract_fn(SUPPORT, "refresh_fan_merch_bundles")
        self.assertNotIn("bundles.set(None)", body)

    def test_push_sync_errors_are_silent(self) -> None:
        """NativePushControl must not set the error signal on push sync failure."""
        # The push sync effect and toggle must not call error.try_set(Some(...))
        # or error.set(Some(...)) for transient failures.
        push_section = FAN_HOME.split("fn NativePushControl", 1)[1]
        # Cut at the next component or function
        next_fn = re.search(r"\n#\[component\]\nfn ", push_section)
        if next_fn:
            push_section = push_section[: next_fn.start()]
        self.assertNotIn(
            "error.try_set(Some(message))",
            push_section,
            "NativePushControl still surfaces push sync errors",
        )

    def test_push_preferences_errors_are_silent(self) -> None:
        """update_push_preference must not set error on write failure."""
        body = extract_fn(FAN_HOME, "update_push_preference")
        self.assertNotIn(
            "error.try_set(Some(",
            body,
            "update_push_preference still surfaces write errors",
        )

    def test_wallet_qr_and_resend_errors_are_silent(self) -> None:
        """Wallet QR toggle and resend must not set error on failure."""
        wallet_card = extract_fn(WALLET, "WalletCard")
        self.assertNotIn(
            "error.set(Some(message))",
            wallet_card,
            "WalletCard resend still surfaces errors",
        )
        ticket_card = extract_fn(WALLET, "WalletTicketCard")
        self.assertNotIn(
            "error.set(Some(message))",
            ticket_card,
            "WalletTicketCard QR still surfaces errors",
        )

    def test_wallet_import_and_claim_use_inline_status(self) -> None:
        """FanWallet import/claim must use inline status, not the shared error."""
        wallet = extract_fn(WALLET, "FanWallet")
        self.assertIn("import_status", wallet, "FanWallet missing import_status signal")
        self.assertIn("claim_status", wallet, "FanWallet missing claim_status signal")
        self.assertIn("inline-form-error", wallet, "FanWallet missing inline-form-error rendering")

    def test_checkout_uses_inline_error(self) -> None:
        """FanTicketSale must use checkout_error signal, not shared error."""
        sale = extract_fn(EVENTS, "FanTicketSale")
        self.assertIn("checkout_error", sale, "FanTicketSale missing checkout_error signal")
        self.assertIn("inline-form-error", sale, "FanTicketSale missing inline-form-error rendering")

    def test_event_interest_toggle_is_silent(self) -> None:
        """FanEventCard interest toggle must not set error on failure."""
        card = extract_fn(EVENTS, "FanEventCard")
        self.assertNotIn(
            "error.set(Some(message))",
            card,
            "FanEventCard interest toggle still surfaces errors",
        )

    def test_external_link_surfaces_errors(self) -> None:
        """ExternalLink must surface URL open failures via the error signal."""
        ext = extract_fn(EVENTS, "ExternalLink")
        self.assertIn(
            "error.set(Some(message))",
            ext,
            "ExternalLink must not silently swallow URL open errors",
        )

    def test_fan_shell_lock_and_share_are_silent(self) -> None:
        """Fan shell lock and share must not set error on failure."""
        shell = SHELL
        # The fan_lock call should use if let, not match with error.set
        lock_section = shell.split("fan_lock", 1)[1][:500] if "fan_lock" in shell else ""
        self.assertNotIn(
            "error.try_set(Some(message))",
            lock_section,
            "Fan shell lock still surfaces errors",
        )
        # Copy/share should not set error
        copy_section = shell.split("copy_text", 1)[1][:300] if "copy_text" in shell else ""
        self.assertNotIn(
            "error.set(Some(message))",
            copy_section,
            "Fan shell copy still surfaces errors",
        )

    def test_inline_form_error_class_exists(self) -> None:
        """styles.css must define the .inline-form-error class."""
        css = (ROOT / "styles.css").read_text(encoding="utf-8")
        self.assertIn(".inline-form-error", css)

    def test_operator_refresh_functions_still_surface_errors(self) -> None:
        """Staff/operator refresh functions must still surface errors."""
        operator_fns = [
            "refresh_operator_events",
            "refresh_operator_qr",
            "refresh_operator_signal",
            "refresh_operator_autopilot",
            "refresh_operator_chief",
            "refresh_operator_ops",
        ]
        at_least_one = False
        for fn_name in operator_fns:
            body = extract_fn(SUPPORT, fn_name)
            if "set_error_debounced(error," in body or "error.set(Some(" in body:
                at_least_one = True
                break
        self.assertTrue(
            at_least_one,
            "Operator refresh functions must still surface errors for staff",
        )

    def test_is_transient_error_helper_exists(self) -> None:
        """The is_transient_error classification helper must exist."""
        self.assertIn("fn is_transient_error", SUPPORT)
        body = extract_fn(SUPPORT, "is_transient_error")
        self.assertIn("timeout", body)
        self.assertIn("network", body)
        self.assertIn("connection", body)
        self.assertIn("offline", body)


if __name__ == "__main__":
    unittest.main()
