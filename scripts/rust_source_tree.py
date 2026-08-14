from __future__ import annotations

from pathlib import Path
import re

_INCLUDE = re.compile(r'include!\("([^"]+)"\);')


def read_rust_module(root: Path, relative: str | Path) -> str:
    """Read a Rust source file as the logical module formed by local include! files."""
    start = root / relative
    seen: set[Path] = set()

    def visit(path: Path) -> str:
        resolved = path.resolve()
        if resolved in seen:
            return ""
        seen.add(resolved)
        text = path.read_text(encoding="utf-8")
        parts = [text]
        for child in _INCLUDE.findall(text):
            target = path.parent / child
            if not target.is_file():
                raise FileNotFoundError(f"broken include! in {path}: {child}")
            parts.append(visit(target))
        return "\n".join(parts)

    return visit(start)
