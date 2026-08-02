#!/usr/bin/env python3
"""Set the bundle version without invalidating Cargo's dependency cache."""

from __future__ import annotations

import argparse
import json
import re
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
SEMVER = re.compile(r"\d+\.\d+\.\d+(?:[-+][0-9A-Za-z.-]+)?")


def normalize_version(raw: str) -> str:
    version = raw.removeprefix("apk-v").removeprefix("v").strip()
    if not SEMVER.fullmatch(version):
        raise ValueError(f"invalid release version: {version}")
    return version


def derive_android_version_code(version: str) -> int:
    stable = version.split("-", 1)[0].split("+", 1)[0]
    major, minor, patch = map(int, stable.split("."))
    if minor >= 1000 or patch >= 1000:
        raise ValueError("minor and patch versions must be below 1000")
    code = major * 1_000_000 + minor * 1_000 + patch
    if not 1 <= code <= 2_100_000_000:
        raise ValueError("derived Android version code is outside the supported range")
    return code


def update_config(
    version: str,
    build_number: str | None,
    android_version_code: str | None,
) -> None:
    config_path = ROOT / "src-tauri" / "tauri.conf.json"
    config = json.loads(config_path.read_text())
    config["version"] = version
    bundle = config.setdefault("bundle", {})
    if build_number:
        if not build_number.isdigit() or len(build_number) > 18:
            raise ValueError("iOS build number must contain 1-18 digits")
        bundle.setdefault("iOS", {})["bundleVersion"] = build_number
    if android_version_code:
        code = (
            derive_android_version_code(version)
            if android_version_code == "auto"
            else int(android_version_code)
        )
        if not 1 <= code <= 2_100_000_000:
            raise ValueError("Android version code must be between 1 and 2100000000")
        bundle.setdefault("android", {})["versionCode"] = code
    config_path.write_text(json.dumps(config, ensure_ascii=False, indent=2) + "\n")


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("version")
    parser.add_argument("--build-number")
    parser.add_argument("--android-version-code")
    args = parser.parse_args()
    try:
        version = normalize_version(args.version)
        update_config(version, args.build_number, args.android_version_code)
    except (ValueError, OSError, json.JSONDecodeError) as error:
        raise SystemExit(str(error)) from error
    print(f"bundle release version set to {version}")


if __name__ == "__main__":
    main()
