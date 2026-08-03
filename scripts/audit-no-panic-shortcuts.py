#!/usr/bin/env python3
from __future__ import annotations

from pathlib import Path
import re
import sys

FORBIDDEN = re.compile(
    r"\.(?:unwrap(?:_or(?:_else|_default)?|_err|_unchecked)?|expect(?:_err)?)\s*\("
)


def rust_files(root: Path):
    for base in (root / "src", root / "src-tauri" / "src"):
        if base.is_dir():
            yield from sorted(base.rglob("*.rs"))
    build = root / "src-tauri" / "build.rs"
    if build.is_file():
        yield build


def main() -> int:
    root = Path(sys.argv[1] if len(sys.argv) > 1 else ".").resolve()
    bad: list[str] = []
    for path in rust_files(root):
        text = path.read_text(encoding="utf-8")
        lines = text.splitlines()
        for match in FORBIDDEN.finditer(text):
            line_number = text.count("\n", 0, match.start()) + 1
            line = lines[line_number - 1].strip() if line_number <= len(lines) else match.group(0)
            bad.append(f"{path.relative_to(root)}:{line_number}: {line}")
    if bad:
        print("Forbidden panic shortcuts found:", file=sys.stderr)
        print("\n".join(bad), file=sys.stderr)
        return 1
    print("OK: no unwrap*/expect* shortcuts in Virya Signal Rust")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
