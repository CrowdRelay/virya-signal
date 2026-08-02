#!/usr/bin/env python3
"""Collect exactly one mobile build artifact and emit its SHA-256 checksum."""

from __future__ import annotations

import argparse
import hashlib
import shutil
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
SEARCH_ROOTS = {
    "apk": ROOT / "src-tauri" / "gen" / "android" / "app" / "build" / "outputs" / "apk",
    "aab": ROOT / "src-tauri" / "gen" / "android" / "app" / "build" / "outputs" / "bundle",
    "ipa": ROOT / "src-tauri" / "gen" / "apple" / "build",
}


def candidates(kind: str) -> list[Path]:
    root = SEARCH_ROOTS[kind]
    if not root.is_dir():
        raise SystemExit(f"artifact directory does not exist: {root.relative_to(ROOT)}")
    result = [
        path
        for path in root.rglob(f"*.{kind}")
        if "androidTest" not in path.parts
        and "unaligned" not in path.name
        and "unsigned" not in path.name
    ]
    if len(result) > 1:
        preferred = [path for path in result if "universal" in path.name]
        if len(preferred) == 1:
            result = preferred
    return sorted(result)


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("kind", choices=sorted(SEARCH_ROOTS))
    parser.add_argument("output", type=Path)
    parser.add_argument(
        "--max-size-mib",
        type=int,
        help="fail if the collected artifact exceeds this size",
    )
    args = parser.parse_args()

    found = candidates(args.kind)
    if len(found) != 1:
        listed = "\n".join(f"  - {path.relative_to(ROOT)}" for path in found) or "  (none)"
        raise SystemExit(f"expected exactly one {args.kind.upper()} artifact, found {len(found)}:\n{listed}")

    output = args.output if args.output.is_absolute() else ROOT / args.output
    size_bytes = found[0].stat().st_size
    if args.max_size_mib is not None:
        limit = args.max_size_mib * 1024 * 1024
        if size_bytes > limit:
            size_mib = size_bytes / 1024 / 1024
            raise SystemExit(
                f"{found[0].name} is {size_mib:.1f} MiB; limit is {args.max_size_mib} MiB"
            )
    output.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(found[0], output)
    digest = sha256(output)
    checksum = output.with_name(f"{output.name}.sha256")
    checksum.write_text(f"{digest}  {output.name}\n")
    print(f"collected {found[0].relative_to(ROOT)} -> {output.relative_to(ROOT)}")
    print(f"size={size_bytes / 1024 / 1024:.1f} MiB")
    print(f"sha256={digest}")


if __name__ == "__main__":
    main()
