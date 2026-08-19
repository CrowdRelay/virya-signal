#!/usr/bin/env python3
from __future__ import annotations
import re
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CSS = (ROOT / "styles.css").read_text(encoding="utf-8")
FAN = (ROOT / "src/app/fan.rs").read_text(encoding="utf-8")
SHELL = (ROOT / "src/app/fan/shell.rs").read_text(encoding="utf-8")
HOME = (ROOT / "src/app/fan_home.rs").read_text(encoding="utf-8")
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
    def test_shared_virya_brand_tokens_match_web_v2(self) -> None:
        expected = {
            "--bg": "#070908", "--bg-raised": "#0b100f", "--surface": "#101715",
            "--surface-2": "#16201d", "--text": "#eef4f1", "--muted": "#98a5a0",
            "--line": "#27322f", "--signal": "#84b4ac", "--signal-hot": "#93c6c0",
            "--signal-deep": "#26655d",
        }
        for name, value in expected.items():
            self.assertEqual(token(name), value, name)

    def test_accessibility_basics(self) -> None:
        self.assertGreaterEqual(int(token("--touch-min").removesuffix("px")), 44)
        bg, surface = token("--bg"), token("--surface")
        for fg in ("--text", "--muted", "--signal", "--signal-hot"):
            self.assertGreaterEqual(contrast(token(fg), bg), 4.5, fg)
            self.assertGreaterEqual(contrast(token(fg), surface), 4.5, fg)
        self.assertIn("prefers-reduced-motion: reduce", CSS)
        self.assertIn("focus-visible", CSS)

    def test_home_is_action_first_not_fandom_analytics(self) -> None:
        self.assertNotIn('class="stats-grid fan-home-stats"', HOME)
        self.assertNotIn('class="participation-history"', HOME)
        self.assertNotIn("signal_snapshot_updated", HOME)
        self.assertNotIn('class="live-dot"', SHELL)

    def test_synesthesia_stays_contextual(self) -> None:
        self.assertIn("synesthesia.started || synesthesia.completed", HOME)
        self.assertNotIn('class="signal-steps"', FAN)

    def test_primary_mobile_nav_stays_small_and_clear(self) -> None:
        self.assertIn('<nav class="bottom-nav four primary-four">', SHELL)
        for tab in ("FanTab::Signal", "FanTab::Events", "FanTab::Merch", "FanTab::Wallet"):
            self.assertIn(tab, SHELL)

    def test_long_lists_keep_rendering_guards(self) -> None:
        self.assertIn("content-visibility: auto", CSS)
        self.assertIn("contain-intrinsic-size", CSS)
        self.assertRegex(CSS, r"@media \(max-width:\s*520px\)")
        self.assertRegex(CSS, r"\.fan-merch-list\s*\{\s*grid-template-columns:\s*1fr")

    def test_logo_uses_shared_core_colors_without_effect_bloat(self) -> None:
        self.assertIn('fill="#070908"', LOGO)
        self.assertGreaterEqual(LOGO.count('fill="#93c6c0"'), 3)
        self.assertNotIn("<filter", LOGO)
        self.assertNotIn("gradient", LOGO.lower())

if __name__ == "__main__":
    unittest.main()
