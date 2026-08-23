#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import zipfile
from pathlib import Path

REQUIRED_PATHS = (
    "/latarnik",
    "/pl/latarnik",
    "/my-signal",
    "/pl/my-signal",
    "/signal/confirm",
    "/pl/signal/confirm",
)


def require_bytes(haystack: bytes, needle: str, label: str) -> None:
    if needle.encode("utf-8") not in haystack:
        raise ValueError(f"AAB merged manifest is missing {label}: {needle}")


def verify(aab: Path, *, tauri_config_path: Path) -> dict[str, object]:
    if not aab.is_file():
        raise ValueError(f"AAB does not exist: {aab}")
    try:
        config = json.loads(tauri_config_path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        raise ValueError(f"could not read Tauri config: {error}") from error
    package_name = str(config.get("identifier") or "").strip()
    if not package_name:
        raise ValueError("tauri.conf.json is missing identifier")

    try:
        with zipfile.ZipFile(aab) as archive:
            manifest_name = "base/manifest/AndroidManifest.xml"
            if manifest_name not in archive.namelist():
                raise ValueError(f"AAB is missing {manifest_name}")
            manifest = archive.read(manifest_name)
    except zipfile.BadZipFile as error:
        raise ValueError("AAB is not a valid ZIP container") from error

    # AAB manifests are protobuf-encoded, but their string table preserves the
    # merged attribute names/values. These checks therefore validate the exact
    # release artifact rather than trusting generated source XML.
    for needle, label in [
        (package_name, "application id"),
        ("android.intent.action.VIEW", "VIEW action"),
        ("android.intent.category.DEFAULT", "DEFAULT category"),
        ("android.intent.category.BROWSABLE", "BROWSABLE category"),
        ("autoVerify", "verified App Link attribute"),
        ("https", "https scheme"),
        ("virya.music", "Virya App Link host"),
        ("virya-signal", "Signal native Synesthesia scheme"),
        ("my-signal", "Signal native Synesthesia host"),
    ]:
        require_bytes(manifest, needle, label)
    for path in REQUIRED_PATHS:
        require_bytes(manifest, path, "required App Link pathPrefix")

    return {"package": package_name, "host": "virya.music", "nativeScheme": "virya-signal", "paths": list(REQUIRED_PATHS)}


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Verify Virya Signal Android App Links in the final Play AAB"
    )
    parser.add_argument("aab", type=Path)
    parser.add_argument(
        "--tauri-config", type=Path, default=Path("src-tauri/tauri.conf.json")
    )
    args = parser.parse_args()
    try:
        report = verify(args.aab, tauri_config_path=args.tauri_config)
    except ValueError as error:
        parser.error(str(error))
    print(
        "SIGNAL_AAB_APP_LINKS_GATE=PASS "
        f"package={report['package']} host={report['host']} paths={len(report['paths'])}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
