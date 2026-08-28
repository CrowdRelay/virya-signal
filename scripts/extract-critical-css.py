#!/usr/bin/env python3
"""Extract critical above-the-fold CSS from styles.css and inline it into index.html.

Critical CSS covers what the user sees before the deferred stylesheet activates:
the boot splash (already inline in index.html), the app shell, topbar, bottom-nav,
skeleton, screen layout, eyebrow text, and button base styles.

This script reads styles.css, extracts the rules matching a curated selector
list, and injects them into a <style id="critical-css"> block in index.html.
Run it after `trunk build` or manually during development to keep the inline
CSS in sync with the source.

Usage:
    python3 scripts/extract-critical-css.py [--check]
    --check: verify the inline CSS in index.html matches the extracted source
"""
from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CSS_PATH = ROOT / "styles.css"
HTML_PATH = ROOT / "index.html"

# Selectors whose rules are critical for first paint. The boot splash has its
# own inline styles in index.html already; these cover the app shell that
# mounts behind it and the skeleton/empty states that replace it.
CRITICAL_SELECTORS = [
    ":root",
    "*",
    "html, body, #root",
    "body",
    "button, input, textarea, select, a",
    "button, input, textarea, select",
    "button, a",
    "button",
    "button:focus-visible, a:focus-visible, summary:focus-visible",
    "a",
    ".app-shell",
    ".authenticated",
    ".topbar",
    ".topbar strong",
    ".topbar-actions",
    ".topbar-actions button",
    ".content",
    ".screen",
    ".screen-title",
    ".screen-title h2",
    ".eyebrow",
    ".bottom-nav",
    ".bottom-nav.three",
    ".bottom-nav.four",
    ".bottom-nav.five",
    ".bottom-nav.six",
    ".bottom-nav button",
    ".bottom-nav button:active",
    ".bottom-nav button.active",
    ".nav-icon",
    ".skeleton-stack",
    ".skeleton-stack i",
    ".tab-page[hidden]",
    ".tab-page.hidden",
    ".primary",
    ".primary:active:not(:disabled)",
    "button:disabled",
    "@supports not (backdrop-filter: blur(12px))",
    "@keyframes skeleton-shimmer",
    "@keyframes fade-in",
    "@keyframes ripple",
    "@keyframes nav-activate",
    "@keyframes tab-enter",
    "@keyframes list-item-enter",
]

# Custom properties from :root that are needed for critical rendering.
# The full :root block is included via the :root selector above.


def extract_rules(css: str) -> str:
    """Extract all rules whose selector matches the critical list."""
    # Parse top-level rules (at-rules and rule blocks).
    rules: list[str] = []
    i = 0
    depth = 0
    start = 0
    in_at_rule = False
    at_rule_header = ""

    while i < len(css):
        char = css[i]
        if char == "{":
            if depth == 0 and in_at_rule:
                # Start of at-rule body
                depth = 1
            elif depth == 0:
                # Start of a normal rule
                depth = 1
            else:
                depth += 1
        elif char == "}":
            depth -= 1
            if depth == 0:
                block = css[start : i + 1]
                selector_part = block[: block.index("{")].strip()
                # Check if this rule's selector matches any critical selector
                if in_at_rule:
                    # Include keyframes and supports that are in the critical list
                    if any(sel in selector_part for sel in CRITICAL_SELECTORS):
                        rules.append(block)
                    in_at_rule = False
                    at_rule_header = ""
                else:
                    # Normal rule — check selector
                    for critical in CRITICAL_SELECTORS:
                        if critical in selector_part:
                            rules.append(block)
                            break
                start = i + 1
        elif char == "@" and depth == 0:
            in_at_rule = True
            at_rule_header = ""
            start = i
        i += 1

    return "\n".join(rules)


def extract_critical_css() -> str:
    """Extract the critical CSS from styles.css."""
    css = CSS_PATH.read_text(encoding="utf-8")
    return extract_rules(css)


def update_index_html(critical_css: str) -> bool:
    """Inject critical CSS into index.html. Returns True if changed."""
    html = HTML_PATH.read_text(encoding="utf-8")
    marker_start = '<style id="critical-css">'
    marker_end = "</style>"

    if marker_start in html:
        # Replace existing block
        pattern = re.compile(
            re.escape(marker_start) + r".*?" + re.escape(marker_end),
            re.DOTALL,
        )
        new_block = f'{marker_start}\n{critical_css}\n    {marker_end}'
        new_html = pattern.sub(new_block, html, count=1)
    else:
        # Insert before the first deferred stylesheet link
        insert_before = '<link data-trunk rel="css" href="styles.css"'
        new_block = f'    {marker_start}\n{critical_css}\n    {marker_end}\n    '
        new_html = html.replace(insert_before, new_block + insert_before, 1)

    if new_html != html:
        HTML_PATH.write_text(new_html, encoding="utf-8")
        return True
    return False


def check_index_html(critical_css: str) -> bool:
    """Verify the inline CSS matches the extracted source."""
    html = HTML_PATH.read_text(encoding="utf-8")
    marker_start = '<style id="critical-css">'
    marker_end = "</style>"
    start = html.find(marker_start)
    if start == -1:
        return False
    end = html.find(marker_end, start)
    if end == -1:
        return False
    inline = html[start + len(marker_start) : end].strip()
    return inline == critical_css.strip()


def main() -> int:
    parser = argparse.ArgumentParser(description="Extract and inline critical CSS")
    parser.add_argument("--check", action="store_true", help="verify inline CSS is current")
    args = parser.parse_args()

    critical = extract_critical_css()

    if args.check:
        if check_index_html(critical):
            print("CRITICAL_CSS=OK")
            return 0
        print("CRITICAL_CSS=STALE")
        return 1

    changed = update_index_html(critical)
    if changed:
        print(f"CRITICAL_CSS=UPDATED ({len(critical)} bytes)")
    else:
        print(f"CRITICAL_CSS=CURRENT ({len(critical)} bytes)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
