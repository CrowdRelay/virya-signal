#!/usr/bin/env python3
import argparse
import gzip
import os
from pathlib import Path

CODE_SUFFIXES = {".wasm", ".js", ".mjs", ".css"}


def kib(size: int) -> float:
    return size / 1024


def inspect(
    dist: Path,
    max_wasm_kib: int,
    max_total_kib: int,
    max_code_gzip_kib: int,
) -> tuple[int, int, int, list[tuple[Path, int]]]:
    if not dist.is_dir():
        raise ValueError(f"frontend output does not exist: {dist}")
    files = sorted((path, path.stat().st_size) for path in dist.rglob("*") if path.is_file())
    if not files:
        raise ValueError("frontend output is empty")

    wasm_size = sum(size for path, size in files if path.suffix == ".wasm")
    total_size = sum(size for _, size in files)
    code = [path for path, _ in files if path.suffix.lower() in CODE_SUFFIXES]
    code_gzip_size = sum(
        len(gzip.compress(path.read_bytes(), compresslevel=9, mtime=0)) for path in code
    )

    if wasm_size == 0:
        raise ValueError("frontend output contains no WASM module")
    if wasm_size > max_wasm_kib * 1024:
        raise ValueError(
            f"WASM size {kib(wasm_size):.1f} KiB exceeds {max_wasm_kib} KiB safety ceiling"
        )
    if total_size > max_total_kib * 1024:
        raise ValueError(
            f"frontend size {kib(total_size):.1f} KiB exceeds {max_total_kib} KiB safety ceiling"
        )
    if code_gzip_size > max_code_gzip_kib * 1024:
        raise ValueError(
            f"compressed code projection {kib(code_gzip_size):.1f} KiB exceeds "
            f"{max_code_gzip_kib} KiB transfer ceiling"
        )
    return wasm_size, total_size, code_gzip_size, files


def wasm_budget_state(wasm_size: int, warn_wasm_kib: int, max_wasm_kib: int) -> str:
    if wasm_size > max_wasm_kib * 1024:
        return "fail"
    if wasm_size > warn_wasm_kib * 1024:
        return "warning"
    return "target"


def main() -> int:
    parser = argparse.ArgumentParser(description="Enforce Virya Signal frontend size budgets")
    parser.add_argument("dist", nargs="?", default="dist", type=Path)
    # Raw WASM is still bounded for parse/memory safety, but ordinary product
    # growth is governed by compare_web_metrics.py against the previous successful
    # main. The transfer-aware compressed-code ceiling is the more meaningful
    # absolute network guard than the old historical 1792 KiB raw limit.
    parser.add_argument("--warn-wasm-kib", type=int, default=2048)
    parser.add_argument("--max-wasm-kib", type=int, default=2304)
    parser.add_argument("--max-total-kib", type=int, default=3072)
    parser.add_argument("--max-code-gzip-kib", type=int, default=896)
    parser.add_argument("--min-hard-headroom-kib", type=int, default=32)
    args = parser.parse_args()

    if (
        args.warn_wasm_kib <= 0
        or args.max_wasm_kib <= 0
        or args.max_total_kib <= 0
        or args.max_code_gzip_kib <= 0
        or args.min_hard_headroom_kib < 0
    ):
        parser.error("size budgets must be positive and reserved headroom cannot be negative")
    if args.warn_wasm_kib >= args.max_wasm_kib:
        parser.error("WASM early-warning budget must stay below the hard limit")

    try:
        wasm_size, total_size, code_gzip_size, files = inspect(
            args.dist,
            args.max_wasm_kib,
            args.max_total_kib,
            args.max_code_gzip_kib,
        )
    except ValueError as error:
        parser.error(str(error))

    largest = sorted(files, key=lambda item: item[1], reverse=True)[:5]
    full_gzip_size = sum(
        len(gzip.compress(path.read_bytes(), compresslevel=9, mtime=0))
        for path, _ in files
    )
    wasm_modules = sum(1 for path, _ in files if path.suffix == ".wasm")
    state = wasm_budget_state(wasm_size, args.warn_wasm_kib, args.max_wasm_kib)
    hard_headroom = args.max_wasm_kib - kib(wasm_size)
    target_delta = kib(wasm_size) - args.warn_wasm_kib

    if hard_headroom < args.min_hard_headroom_kib:
        raise SystemExit(
            f"WASM reserved headroom {hard_headroom:.1f} KiB is below "
            f"{args.min_hard_headroom_kib} KiB; keep emergency release capacity below "
            f"the {args.max_wasm_kib} KiB absolute safety ceiling"
        )

    print(
        f"frontend size: {kib(total_size):.1f} KiB; WASM: {kib(wasm_size):.1f} KiB; "
        f"code gzip projection: {kib(code_gzip_size):.1f} KiB; "
        f"full gzip projection: {kib(full_gzip_size):.1f} KiB; "
        f"WASM modules: {wasm_modules}; budget state: {state}"
    )

    if state == "warning":
        warning = (
            f"WASM is {target_delta:.1f} KiB above the {args.warn_wasm_kib} KiB early-warning "
            f"target; {hard_headroom:.1f} KiB remains before the {args.max_wasm_kib} KiB safety ceiling"
        )
        print(f"WASM_SIZE_WARNING={warning}")
        if os.environ.get("GITHUB_ACTIONS") == "true":
            print(f"::warning title=Virya Signal WASM headroom::{warning}")

    for path, size in largest:
        print(f"  {kib(size):8.1f} KiB  {path.relative_to(args.dist)}")

    summary_path = os.environ.get("GITHUB_STEP_SUMMARY")
    if summary_path:
        with Path(summary_path).open("a", encoding="utf-8") as summary:
            summary.write("## Virya Signal frontend size\n\n")
            summary.write(f"- Total raw: **{kib(total_size):.1f} KiB**\n")
            summary.write(f"- WASM raw: **{kib(wasm_size):.1f} KiB**\n")
            summary.write(f"- Code gzip projection: **{kib(code_gzip_size):.1f} KiB**\n")
            summary.write(f"- Compressed-code ceiling: **{args.max_code_gzip_kib} KiB**\n")
            summary.write(f"- Raw WASM warning: **{args.warn_wasm_kib} KiB**\n")
            summary.write(f"- Raw WASM safety ceiling: **{args.max_wasm_kib} KiB**\n")
            summary.write(f"- Raw WASM headroom: **{hard_headroom:.1f} KiB**\n")
            summary.write(f"- Reserved emergency headroom: **{args.min_hard_headroom_kib} KiB**\n")
            summary.write(f"- Full gzip projection: **{kib(full_gzip_size):.1f} KiB**\n")
            summary.write(f"- WASM modules/chunks: **{wasm_modules}**\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
