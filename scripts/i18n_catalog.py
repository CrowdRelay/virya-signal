"""Build-time helpers for the static Virya Signal PL/EN Rust catalogs."""
from __future__ import annotations

import re
from pathlib import Path

_ENTRY = re.compile(r'^\s*"([^"]+)"\s*=>\s*"([^"]*)",\s*$', re.MULTILINE)


def parse_rust_catalog(path: Path) -> dict[str, str]:
    source = path.read_text(encoding="utf-8")
    entries = _ENTRY.findall(source)
    keys = [key for key, _ in entries]
    if len(keys) != len(set(keys)):
        duplicates = sorted({key for key in keys if keys.count(key) > 1})
        raise ValueError(f"duplicate translation keys in {path}: {duplicates}")
    if not entries:
        raise ValueError(f"empty translation catalog: {path}")
    return dict(entries)


def load_catalog_pair(root: Path) -> tuple[dict[str, str], dict[str, str]]:
    pl = parse_rust_catalog(root / "src/i18n/pl.rs")
    en = parse_rust_catalog(root / "src/i18n/en.rs")
    if set(pl) != set(en):
        missing_en = sorted(set(pl) - set(en))
        missing_pl = sorted(set(en) - set(pl))
        raise ValueError(
            f"PL/EN key mismatch: missing_en={missing_en}, missing_pl={missing_pl}"
        )
    return pl, en
