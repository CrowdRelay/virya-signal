#!/usr/bin/env python3
"""The responsive gate must stay a rendered proof, not a CSS opinion.

Measured on the built bundle at a 320px viewport with a 72-character display
name, before the fix: ten elements ran past the viewport and the fan menu
button sat at right: 1248px. The shell is `height: 100dvh; overflow: hidden`,
so `document.scrollWidth` stayed 320 the whole time — the overflow was silent
clipping, and the primary navigation control was simply unreachable.

These assertions keep the three things that make that catchable: the gate runs
automatically, it measures the widths that matter, and the stylesheet keeps the
declarations that let server-provided words break instead of the layout.
"""

from __future__ import annotations

import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
GATE = ROOT / "scripts/check-responsive.mjs"
CHECK = ROOT / ".github/workflows/check.yml"
STYLES = ROOT / "styles.css"

# The narrow end is the one that breaks; 320 is the smallest width Android
# still ships (small-screen phones in landscape-locked kiosk mode included).
REQUIRED_WIDTHS = (320, 360, 375, 390, 430, 768, 1024, 1280, 1440)


class ResponsiveGateRuns(unittest.TestCase):
    def test_the_gate_is_wired_into_the_always_on_check(self) -> None:
        # Lighthouse Watch is workflow_dispatch only, so a gate parked there
        # would never run. This one belongs where every push and PR goes.
        workflow = CHECK.read_text(encoding="utf-8")
        self.assertIn("responsive:", workflow)
        self.assertIn("node scripts/check-responsive.mjs", workflow)
        self.assertIn("name: virya-signal-web-dist", workflow)
        self.assertIn("scripts/ui-preview/serve.sh", workflow)

    def test_the_gate_measures_every_width_that_matters(self) -> None:
        gate = GATE.read_text(encoding="utf-8")
        for width in REQUIRED_WIDTHS:
            self.assertIn(str(width), gate, f"width {width} is not covered")

    def test_the_gate_does_not_trust_document_scroll_width(self) -> None:
        # `scrollWidth` cannot see this bug at all. The check has to compare
        # every box against the viewport and only excuse a genuinely scrollable
        # ancestor.
        gate = GATE.read_text(encoding="utf-8")
        self.assertIn("getBoundingClientRect", gate)
        self.assertIn("overflowX", gate)
        self.assertIn("clientWidth", gate)

    def test_the_gate_stresses_server_provided_text(self) -> None:
        # Without this pass the gate reports a clean sweep on copy that happens
        # to contain spaces, which is not the case that broke.
        gate = GATE.read_text(encoding="utf-8")
        self.assertIn("SERVER_TEXT_SELECTORS", gate)
        self.assertIn(".topbar strong", gate)
        self.assertIn("'a'.repeat(72)", gate)

    def test_the_gate_checks_touch_targets_and_input_font_size(self) -> None:
        gate = GATE.read_text(encoding="utf-8")
        self.assertIn("TOUCH_MIN = 44", gate)
        # Under 16px, iOS Safari zooms the page on focus and the user has to
        # undo the layout by hand.
        self.assertIn("INPUT_FONT_MIN = 16", gate)


class LayoutSurvivesServerStrings(unittest.TestCase):
    def test_the_topbar_can_never_push_its_own_controls_off_screen(self) -> None:
        styles = STYLES.read_text(encoding="utf-8")
        self.assertIn(".topbar > div { min-width: 0; }", styles)
        self.assertIn(".topbar strong, .topbar .eyebrow { overflow-wrap: anywhere; }", styles)
        # The actions column holds the only way into the menu, so it must keep
        # its intrinsic width whatever the name beside it does.
        topbar_actions = styles.split(".topbar-actions {", 1)[1].split("}", 1)[0]
        self.assertIn("flex-shrink: 0", topbar_actions)

    def test_server_text_surfaces_break_words_rather_than_layout(self) -> None:
        styles = STYLES.read_text(encoding="utf-8")
        for selector in (".hero-card h3", ".fan-coupon strong", ".referral-code-copy"):
            self.assertIn(selector, styles, selector)
        self.assertGreaterEqual(styles.count("overflow-wrap: anywhere"), 3)

    def test_the_language_switch_keeps_a_real_touch_target(self) -> None:
        # `minmax(0, 1fr)` let both buttons shrink to 41px beside a long label.
        styles = STYLES.read_text(encoding="utf-8")
        rule = styles.split(".language-switch {", 1)[1].split("}", 1)[0]
        self.assertIn("min-width: 96px", rule)


if __name__ == "__main__":
    unittest.main()
