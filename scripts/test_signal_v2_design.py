#!/usr/bin/env python3
from __future__ import annotations

import re
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CSS = (ROOT / "styles.css").read_text(encoding="utf-8")
FAN = (ROOT / "src/app/fan.rs").read_text(encoding="utf-8")
SHELL = (ROOT / "src/app/fan/shell.rs").read_text(encoding="utf-8")
GEN = (ROOT / "scripts/generate-crowdrelay-fan-contract.py").read_text(encoding="utf-8")
PUSH = (ROOT / "crates/virya-signal-contracts/src/push.rs").read_text(encoding="utf-8")
LOGO = (ROOT / "branding/signal-v2.svg").read_text(encoding="utf-8")


def token(name: str) -> str:
    match = re.search(rf"(?m)^  {re.escape(name)}:\s*([^;]+);$", CSS)
    if not match:
        raise AssertionError(f"missing CSS token {name}")
    return match.group(1).strip()


def luminance(value: str) -> float:
    rgb = [int(value[i:i+2], 16) / 255 for i in (1, 3, 5)]
    linear = [c / 12.92 if c <= .04045 else ((c + .055) / 1.055) ** 2.4 for c in rgb]
    return .2126 * linear[0] + .7152 * linear[1] + .0722 * linear[2]


def contrast(a: str, b: str) -> float:
    la, lb = luminance(a), luminance(b)
    return (max(la, lb) + .05) / (min(la, lb) + .05)


class SignalV2DesignContract(unittest.TestCase):
    def test_exact_virya_v2_core_tokens(self) -> None:
        expected = {
            "--bg": "#070908",
            "--bg-raised": "#0b100f",
            "--surface": "#101715",
            "--surface-2": "#16201d",
            "--text": "#eef4f1",
            "--muted": "#98a5a0",
            "--line": "#27322f",
            "--signal": "#84b4ac",
            "--signal-hot": "#93c6c0",
            "--signal-deep": "#26655d",
            "--warning": "#f3c51a",
            "--danger": "#e73535",
            "--success": "#70db91",
        }
        for name, value in expected.items():
            self.assertEqual(token(name), value, name)
        self.assertEqual(token("--control-radius"), "6px")

    def test_core_contrast_has_real_headroom(self) -> None:
        bg = token("--bg")
        surface = token("--surface")
        for foreground in ("--text", "--muted", "--signal", "--signal-hot", "--warning", "--success"):
            self.assertGreaterEqual(contrast(token(foreground), bg), 4.5, foreground)
            self.assertGreaterEqual(contrast(token(foreground), surface), 4.5, foreground)

    def test_mobile_accessibility_primitives(self) -> None:
        self.assertEqual(token("--touch-min"), "44px")
        self.assertIn("prefers-reduced-motion: reduce", CSS)
        self.assertIn("outline: 2px solid var(--signal-hot)", CSS)
        self.assertNotIn("input::placeholder, textarea::placeholder { color: #555;", CSS)

    def test_no_decorative_watermark_v_or_duplicate_dashboard(self) -> None:
        self.assertNotRegex(CSS, r'content:\s*"V"')
        self.assertNotIn("compact-referral-hero", SHELL)
        self.assertNotIn("signal-dashboard-hero", SHELL)
        self.assertNotIn('class="signal-steps"', FAN)

    def test_native_model_tests_do_not_reexport_runtime_push_contracts(self) -> None:
        native_tests = (ROOT / "src-tauri/src/models/tests.rs").read_text(encoding="utf-8")
        self.assertNotIn("pub use virya_signal_contracts::push::*;", native_tests)

    def test_pending_referral_count_is_wire_only_after_debloat(self) -> None:
        models = (ROOT / "src/models.rs").read_text(encoding="utf-8")
        self.assertIn("pub pending_referrals: u32", models)
        self.assertIn(
            "CrowdRelay wire parity; V2 Home intentionally does not render pending-referral KPI.",
            models,
        )
        self.assertNotIn("pending_referrals", SHELL)

    def test_referral_debloat_keeps_one_real_copy_target(self) -> None:
        self.assertNotIn("compact-referral-hero", SHELL)
        self.assertNotIn("signal-dashboard-hero", SHELL)
        self.assertEqual(SHELL.count('class="referral-code-copy"'), 1)
        self.assertIn("bridge::copy_text(&url)", SHELL)
        self.assertIn('format!("https://www.virya.music/r/{referral_code}")', SHELL)

    def test_mobile_nav_is_flat_and_signal_led(self) -> None:
        active = re.search(r"(?m)^\.bottom-nav button\.active\s*\{([^}]*)\}", CSS)
        self.assertIsNotNone(active)
        body = active.group(1)
        self.assertIn("var(--signal-hot)", body)
        self.assertIn("inset 0 2px 0", body)
        self.assertNotIn("linear-gradient", body)

    def test_performance_primitives_survive_visual_polish(self) -> None:
        self.assertIn("content-visibility: auto", CSS)
        self.assertIn("contain-intrinsic-size", CSS)
        self.assertIn("@media (max-width: 520px)", CSS)
        self.assertIn(".fan-merch-list { grid-template-columns: 1fr; }", CSS)

    def test_signal_logo_uses_the_same_v2_system(self) -> None:
        self.assertIn('fill="#070908"', LOGO)
        self.assertEqual(LOGO.count('fill="#93c6c0"'), 3)
        self.assertNotIn("<filter", LOGO)
        self.assertNotIn("gradient", LOGO.lower())
        logo = re.search(r"(?m)^\.signal-logo\s*\{([^}]*)\}", CSS)
        self.assertIsNotNone(logo)
        body = logo.group(1)
        self.assertIn("background: #070908", body)
        self.assertIn("border-radius: 6px", body)
        self.assertIn("box-shadow: none", body)
        self.assertNotIn("radial-gradient", body)
        bars = re.search(r"(?m)^\.signal-logo span\s*\{([^}]*)\}", CSS)
        self.assertIsNotNone(bars)
        bars_body = bars.group(1)
        self.assertIn("background: var(--signal-hot)", bars_body)
        self.assertIn("box-shadow: none", bars_body)

    def test_generated_push_contract_is_canonical_and_narrow(self) -> None:
        self.assertIn('"FanPushPreferences:"', GEN)
        self.assertIn('"FanPushPreferencesUpdate:"', GEN)
        self.assertIn("pub struct FanPushPreferencesUpdate", GEN)
        self.assertIn("deny_unknown_fields", GEN)
        self.assertIn(
            "pub use crate::fan::{FanPushPreferences, FanPushPreferencesUpdate};",
            PUSH,
        )


if __name__ == "__main__":
    unittest.main()
