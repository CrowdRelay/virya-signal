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
    ".bottom-nav button.active",
    ".nav-icon",
    ".skeleton-stack",
    ".skeleton-stack i",
    ".tab-page[hidden]",
    ".tab-page.hidden",
    ".primary",
    ".primary:active:not(:disabled)",
    "button:disabled",
    "@keyframes skeleton-shimmer",
    "@keyframes fade-in",
    "@keyframes ripple",
    "@keyframes list-item-enter",
]
# Entries that match nothing are dead weight and hide the fact that a rule was
# renamed, so `--check` fails on them. `.bottom-nav button:active` and
# `@keyframes nav-activate` were already gone from styles.css; `tab-enter`
# lives inside a media query, and only top-level rules are eligible here.

# Custom properties from :root that are needed for critical rendering.
# The full :root block is included via the :root selector above.


def normalize_selector(selector: str) -> str:
    """Collapse whitespace so `a,  b` and `a, b` compare equal."""
    return re.sub(r"\s+", " ", selector.strip())


def extract_rules(css: str) -> tuple[str, list[str]]:
    """Extract the rules whose selector is exactly one of the critical ones.

    Returns the CSS text and the critical selectors that matched nothing.

    The match is on the whole normalized selector, not a substring of it. It
    used to be `critical in selector_part`, and the list contains `a` and
    `button` — so every selector carrying the letter `a` (`.fan-event-card`,
    `.wallet-card`, `@media …`) qualified. 111 KiB of a 123 KiB stylesheet
    ended up inlined in index.html, parsed once from the document and again
    from the deferred sheet, which is the opposite of what critical CSS is for.
    """
    # Comments are stripped first. A comment sitting between two rules would
    # otherwise be read as part of the next rule's selector, so a documented
    # rule silently stopped qualifying as critical — which is what happened to
    # `.content` and every `@keyframes` block on this list.
    css = re.sub(r"/\*.*?\*/", "", css, flags=re.DOTALL)
    wanted = {normalize_selector(sel) for sel in CRITICAL_SELECTORS}
    matched: set[str] = set()
    rules: list[str] = []
    i = 0
    depth = 0
    start = 0

    while i < len(css):
        char = css[i]
        if char == "{":
            depth = 1 if depth == 0 else depth + 1
        elif char == "}":
            depth -= 1
            if depth == 0:
                block = css[start : i + 1]
                selector_part = normalize_selector(block[: block.index("{")])
                # An at-rule's own body carries nested selectors this top-level
                # pass never sees, so only at-rules named outright (the
                # keyframes above) qualify — a `@media` block is not critical
                # unless somebody adds its exact header to the list.
                if selector_part in wanted:
                    matched.add(selector_part)
                    rules.append(block)
                start = i + 1
        i += 1

    unmatched = [sel for sel in CRITICAL_SELECTORS if normalize_selector(sel) not in matched]
    return "\n".join(rules), unmatched


def extract_critical_css() -> tuple[str, list[str]]:
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

    critical, unmatched = extract_critical_css()

    # A selector that matches nothing is either a typo or a rule that has been
    # renamed since. Either way the inline block quietly lost a rule it was
    # supposed to carry, so this is a failure, not a warning.
    if unmatched:
        print("CRITICAL_CSS=UNMATCHED " + ", ".join(unmatched))
        return 1

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
