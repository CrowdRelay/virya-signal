#!/usr/bin/env python3
import argparse
import gzip
import os
from pathlib import Path


def kib(size: int) -> float:
    return size / 1024


def inspect(dist: Path, max_wasm_kib: int, max_total_kib: int) -> tuple[int, int, list[tuple[Path, int]]]:
    if not dist.is_dir():
        raise ValueError(f"frontend output does not exist: {dist}")
    files = sorted((path, path.stat().st_size) for path in dist.rglob("*") if path.is_file())
    if not files:
        raise ValueError(f"frontend output is empty: {dist}")
    wasm_size = sum(size for path, size in files if path.suffix == ".wasm")
    total_size = sum(size for _, size in files)
    if wasm_size == 0:
        raise ValueError("frontend output contains no WASM module")
    if wasm_size > max_wasm_kib * 1024:
        raise ValueError(
            f"WASM size {kib(wasm_size):.1f} KiB exceeds {max_wasm_kib} KiB budget"
        )
    if total_size > max_total_kib * 1024:
        raise ValueError(
            f"frontend size {kib(total_size):.1f} KiB exceeds {max_total_kib} KiB budget"
        )
    return wasm_size, total_size, files


def main() -> int:
    parser = argparse.ArgumentParser(description="Enforce Virya Signal frontend size budgets")
    parser.add_argument("dist", nargs="?", default="dist", type=Path)
    parser.add_argument("--max-wasm-kib", type=int, default=1536)
    parser.add_argument("--max-total-kib", type=int, default=2048)
    args = parser.parse_args()
    if args.max_wasm_kib <= 0 or args.max_total_kib <= 0:
        parser.error("size budgets must be positive")
    try:
        wasm_size, total_size, files = inspect(
            args.dist, args.max_wasm_kib, args.max_total_kib
        )
    except ValueError as error:
        parser.error(str(error))
    largest = sorted(files, key=lambda item: item[1], reverse=True)[:5]
    gzip_size = sum(len(gzip.compress(path.read_bytes(), compresslevel=9)) for path, _ in files)
    wasm_modules = sum(1 for path, _ in files if path.suffix == ".wasm")
    print(
        f"frontend size: {kib(total_size):.1f} KiB; WASM: {kib(wasm_size):.1f} KiB; "
        f"gzip projection: {kib(gzip_size):.1f} KiB; WASM modules: {wasm_modules}"
    )
    for path, size in largest:
        print(f"  {kib(size):8.1f} KiB  {path.relative_to(args.dist)}")
    summary_path = os.environ.get("GITHUB_STEP_SUMMARY")
    if summary_path:
        with Path(summary_path).open("a", encoding="utf-8") as summary:
            summary.write("## Virya Signal frontend size\n\n")
            summary.write(f"- Total: **{kib(total_size):.1f} KiB**\n")
            summary.write(f"- WASM: **{kib(wasm_size):.1f} KiB**\n")
            summary.write(f"- Gzip projection: **{kib(gzip_size):.1f} KiB**\n")
            summary.write(f"- WASM modules/chunks: **{wasm_modules}**\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
