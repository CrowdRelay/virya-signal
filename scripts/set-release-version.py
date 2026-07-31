#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import re
from pathlib import Path

parser = argparse.ArgumentParser()
parser.add_argument("version")
parser.add_argument("--build-number")
parser.add_argument("--android-version-code", type=int)
args = parser.parse_args()
version = args.version.removeprefix("v").strip()
if not re.fullmatch(r"\d+\.\d+\.\d+(?:[-+][0-9A-Za-z.-]+)?", version):
    raise SystemExit(f"invalid release version: {version}")

root = Path(__file__).resolve().parents[1]
for relative in ["Cargo.toml", "src-tauri/Cargo.toml"]:
    path = root / relative
    text = path.read_text()
    text, count = re.subn(
        r'(?m)^(version\s*=\s*")[^"]+("\s*)$',
        rf'\g<1>{version}\g<2>',
        text,
        count=1,
    )
    if count != 1:
        raise SystemExit(f"could not update package version in {relative}")
    path.write_text(text)

config_path = root / "src-tauri" / "tauri.conf.json"
config = json.loads(config_path.read_text())
config["version"] = version
if args.build_number:
    config.setdefault("bundle", {}).setdefault("iOS", {})["bundleVersion"] = str(args.build_number)
if args.android_version_code is not None:
    if not 1 <= args.android_version_code <= 2_100_000_000:
        raise SystemExit("Android version code must be between 1 and 2100000000")
    config.setdefault("bundle", {}).setdefault("android", {})["versionCode"] = args.android_version_code
config_path.write_text(json.dumps(config, ensure_ascii=False, indent=2) + "\n")
print(f"release version set to {version}")
