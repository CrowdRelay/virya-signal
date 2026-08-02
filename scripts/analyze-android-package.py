#!/usr/bin/env python3
from __future__ import annotations

import argparse
import os
import struct
import zipfile
from pathlib import Path


KNOWN_ABIS = {"arm64-v8a", "armeabi-v7a", "x86", "x86_64"}


def elf_load_alignments(payload: bytes) -> list[int]:
    if len(payload) < 64 or payload[:4] != b"\x7fELF":
        raise ValueError("native library is not a valid ELF file")
    if payload[4] != 2 or payload[5] != 1:
        raise ValueError("native library is not a little-endian ELF64 file")
    program_offset = struct.unpack_from("<Q", payload, 32)[0]
    entry_size = struct.unpack_from("<H", payload, 54)[0]
    entry_count = struct.unpack_from("<H", payload, 56)[0]
    if entry_size < 56 or program_offset + entry_size * entry_count > len(payload):
        raise ValueError("native library has an invalid program header table")
    alignments = []
    for index in range(entry_count):
        offset = program_offset + index * entry_size
        if struct.unpack_from("<I", payload, offset)[0] == 1:
            alignments.append(struct.unpack_from("<Q", payload, offset + 48)[0])
    if not alignments:
        raise ValueError("native library has no loadable ELF segments")
    return alignments


def analyze(
    path: Path,
    required_abi: str | None = None,
    required_page_size: int | None = None,
) -> dict[str, object]:
    if not path.is_file():
        raise ValueError(f"Android package does not exist: {path}")
    try:
        with zipfile.ZipFile(path) as archive:
            corrupt = archive.testzip()
            if corrupt:
                raise ValueError(f"Android package contains a corrupt entry: {corrupt}")
            entries = [entry for entry in archive.infolist() if not entry.is_dir()]
            native_libraries = [entry for entry in entries if entry.filename.endswith(".so")]
            if required_page_size:
                for entry in native_libraries:
                    alignments = elf_load_alignments(archive.read(entry))
                    if any(alignment < required_page_size for alignment in alignments):
                        raise ValueError(
                            f"{entry.filename} has ELF LOAD alignment below {required_page_size}"
                        )
    except zipfile.BadZipFile as error:
        raise ValueError("Android package is not a valid ZIP container") from error
    if not entries:
        raise ValueError("Android package is empty")
    abis = {
        part
        for entry in entries
        for part in Path(entry.filename).parts
        if part in KNOWN_ABIS
    }
    if required_abi and abis != {required_abi}:
        raise ValueError(
            f"expected only {required_abi} native libraries, found {sorted(abis) or 'none'}"
        )
    return {
        "file_bytes": path.stat().st_size,
        "compressed_bytes": sum(entry.compress_size for entry in entries),
        "uncompressed_bytes": sum(entry.file_size for entry in entries),
        "abis": sorted(abis),
        "native_library_count": len(native_libraries),
        "largest": sorted(entries, key=lambda entry: entry.file_size, reverse=True)[:8],
        "categories": {
            "native": sum(entry.file_size for entry in native_libraries),
            "dex": sum(entry.file_size for entry in entries if entry.filename.endswith(".dex")),
            "web_assets": sum(
                entry.file_size
                for entry in entries
                if "/assets/" in f"/{entry.filename}" or entry.filename.startswith("assets/")
            ),
            "resources": sum(
                entry.file_size
                for entry in entries
                if entry.filename == "resources.arsc" or entry.filename.startswith("res/")
            ),
        },
    }


def mib(size: int) -> float:
    return size / 1024 / 1024


def main() -> int:
    parser = argparse.ArgumentParser(description="Validate and report APK/AAB contents")
    parser.add_argument("package", type=Path)
    parser.add_argument("--require-abi", choices=sorted(KNOWN_ABIS))
    parser.add_argument("--require-page-size", type=int)
    args = parser.parse_args()
    try:
        report = analyze(args.package, args.require_abi, args.require_page_size)
    except ValueError as error:
        parser.error(str(error))
    print(
        f"Android package: {mib(report['file_bytes']):.1f} MiB on disk; "
        f"{mib(report['uncompressed_bytes']):.1f} MiB installed payload; "
        f"ABIs={','.join(report['abis'])}; native-libs={report['native_library_count']}"
    )
    for entry in report["largest"]:
        print(f"  {mib(entry.file_size):8.1f} MiB  {entry.filename}")
    print("Package categories:")
    for category, size in report["categories"].items():
        print(f"  {mib(size):8.1f} MiB  {category}")
    summary_path = os.environ.get("GITHUB_STEP_SUMMARY")
    if summary_path:
        with Path(summary_path).open("a", encoding="utf-8") as summary:
            summary.write("## Android package\n\n")
            summary.write(f"- Download: **{mib(report['file_bytes']):.1f} MiB**\n")
            summary.write(
                f"- Uncompressed payload: **{mib(report['uncompressed_bytes']):.1f} MiB**\n"
            )
            summary.write(f"- Native ABIs: **{', '.join(report['abis'])}**\n")
            summary.write(f"- Native libraries: **{report['native_library_count']}**\n")
            summary.write("\n### Payload categories\n\n")
            for category, size in report["categories"].items():
                summary.write(f"- {category}: **{mib(size):.1f} MiB**\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
