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

def resolve_colour(value: str) -> str | None:
    """Resolve a border declaration to a hex colour.

    Handles the two forms this stylesheet uses: a bare `var(--token)` and
    `color-mix(in srgb, <colour> <percent>%, <colour>)`. Returns None for
    anything else so the caller can treat it as unverifiable rather than pass.
    """
    value = value.strip().removeprefix("1px solid ").strip()
    mix = re.fullmatch(
        r"color-mix\(in srgb, *(\S+) *(\d+)%, *(.+?)\)", value
    )
    if mix:
        first, weight, second = resolve_colour(mix.group(1)), int(mix.group(2)), resolve_colour(mix.group(3))
        if first is None or second is None:
            return None
        share = weight / 100
        channels = [
            round(int(first.lstrip("#")[i:i + 2], 16) * share
                  + int(second.lstrip("#")[i:i + 2], 16) * (1 - share))
            for i in (0, 2, 4)
        ]
        return "#" + "".join(f"{c:02x}" for c in channels)
    var = re.fullmatch(r"var\((--[a-z0-9-]+)\)", value)
    if var:
        return token(var.group(1))
    return value if re.fullmatch(r"#[0-9a-fA-F]{6}", value) else None


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

    def test_control_borders_stay_visible(self) -> None:
        """A flat surface has no depth cue, so the border is the whole affordance.

        This is checked because it already regressed once: the move off
        glassmorphism kept the border colours that blur and translucency used to
        supplement, leaving every boundary near 1.3:1 against its own surface
        while body text sat at 17.9:1. Text contrast was never the problem, so a
        text-only check did not notice.

        WCAG 1.4.11 asks 3:1 where a border is the only thing marking a control.
        """
        # Checked against every surface a control can sit on, not just
        # `--surface`. A single-surface check passed at 3.11:1 while the same
        # border sat at 2.86:1 on `--surface-2` (the settings-row hover state)
        # and 2.67:1 on `--surface-3` — the check said compliant and the screen
        # was not.
        surface = token("--surface")
        for name in ("--bg", "--bg-raised", "--surface", "--surface-2", "--surface-3"):
            self.assertGreaterEqual(
                contrast(token("--border-strong"), token(name)),
                3.0,
                f"--border-strong marks controls; below 3:1 on {name} a field stops reading as a field",
            )
        # Dividers are decorative and sit lower on purpose, but must still be
        # separable from the surface they divide.
        self.assertGreaterEqual(
            contrast(token("--border-subtle"), surface),
            1.4,
            "--border-subtle is invisible against its own surface",
        )

    def test_every_control_border_clears_three_to_one(self) -> None:
        """`--line` is a shared brand token pinned at 1.37:1 against `--surface`.

        That is fine for a divider and not fine for a control, where the border
        is the whole affordance (WCAG 1.4.11). Raising `--line` is a brand
        decision that has to land in the web app at the same time, so controls
        take `--border-strong` instead and the divider keeps its value.

        The check resolves what the border actually renders as rather than
        banning a token by name: a `color-mix()` that is mostly gold is a
        visible border even though `--line` appears in it.
        """
        surface = token("--surface")
        # The element/class name has to stand on its own: `.selected` and
        # `.city-picker-results` are not controls, and matching `select` inside
        # them made the check report cards it has nothing to say about.
        control_rules = re.findall(
            r"^([^{}\n]*(?<![\w-])(?:button|input|textarea|select)(?![\w-])[^{}\n]*)\{([^}]*)\}",
            CSS,
            re.MULTILINE,
        )
        offenders = []
        for selector, body in control_rules:
            # A disabled control is exempt: 1.4.11 does not apply to it, and
            # the muted border is what marks it as disabled. `:active` is the
            # transient press state, not the resting affordance a fan scans.
            if ":disabled" in selector or ":active" in selector:
                continue
            for match in re.finditer(r"border(?:-color)?: *([^;]+)", body):
                colour = resolve_colour(match.group(1))
                # `transparent`, `currentColor` and gradients do not resolve to
                # a fixed pair, so they are not judged here.
                if colour is not None and contrast(colour, surface) < 3.0:
                    offenders.append(f"{selector.strip()} -> {match.group(1).strip()}")
        self.assertEqual(offenders, [], "control borders must reach 3:1 against their surface")

    def test_hover_is_gated_behind_a_pointer_that_can_hover(self) -> None:
        """A tapped element keeps :hover until something else is tapped.

        Unwrapped, that leaves a field looking focused when it is not — which on
        a touch-first client is every field the user has ever touched.
        """
        if ":hover" in CSS:
            self.assertIn(
                "@media (hover: hover)",
                CSS,
                "hover styles must be gated on a pointer that can actually hover",
            )

    def test_text_inputs_do_not_trigger_focus_zoom(self) -> None:
        """Under 16px, a mobile browser zooms the page when a field takes focus,
        and leaves the layout scrolled sideways afterwards."""
        match = re.search(r"input, textarea, select \{[^}]*?font-size: *([0-9.]+)(px|rem)", CSS)
        self.assertIsNotNone(match, "could not read the control font size")
        assert match is not None
        size = float(match.group(1)) * (16 if match.group(2) == "rem" else 1)
        self.assertGreaterEqual(size, 16.0, "a control below 16px makes the page zoom on focus")

    def test_home_is_action_first_not_fandom_analytics(self) -> None:
        self.assertNotIn('class="stats-grid fan-home-stats"', HOME)
        self.assertNotIn('class="participation-history"', HOME)
        self.assertNotIn("signal_snapshot_updated", HOME)
        self.assertNotIn('class="live-dot"', SHELL)

    def test_synesthesia_stays_contextual(self) -> None:
        self.assertIn("synesthesia.started || synesthesia.completed", HOME)
        self.assertNotIn('class="signal-steps"', FAN)

    def test_primary_mobile_nav_stays_small_and_clear(self) -> None:
        # Five is the ceiling. Profile joined the bar because account,
        # notification and language settings behind a hamburger is where a fan
        # looks last; AREA stays in the overflow menu as the opt-in side game.
        self.assertIn('<nav class="bottom-nav five"', SHELL)
        self.assertNotIn('class="bottom-nav six', SHELL)
        for tab in (
            "FanTab::Signal",
            "FanTab::Events",
            "FanTab::Merch",
            "FanTab::Wallet",
            "FanTab::Profile",
        ):
            self.assertIn(tab, SHELL)

    def test_long_lists_keep_rendering_guards(self) -> None:
        self.assertIn("content-visibility: auto", CSS)
        self.assertIn("contain-intrinsic-size", CSS)
        self.assertRegex(CSS, r"@media \(max-width:\s*520px\)")
        self.assertRegex(
            CSS,
            r"\.fan-merch-list\s*\{[^}]*grid-template-columns:\s*repeat\(2,\s*minmax\(0,\s*1fr\)\)",
        )

    def test_logo_uses_shared_core_colors_without_effect_bloat(self) -> None:
        self.assertIn('fill="#070908"', LOGO)
        self.assertGreaterEqual(LOGO.count('fill="#93c6c0"'), 3)
        self.assertNotIn("<filter", LOGO)
        self.assertNotIn("gradient", LOGO.lower())

if __name__ == "__main__":
    unittest.main()
