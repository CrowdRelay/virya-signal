#!/usr/bin/env python3
"""Fail closed unless virya.music currently delegates App Links to Virya Signal."""
from __future__ import annotations

import argparse
import json
import re
import urllib.error
import urllib.request

FINGERPRINT = re.compile(r"^(?:[0-9A-F]{2}:){31}[0-9A-F]{2}$")
MAX_BYTES = 256 * 1024


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--url", default="https://virya.music/.well-known/assetlinks.json")
    parser.add_argument("--package", default="music.virya.signal")
    parser.add_argument("--timeout", type=float, default=10.0)
    args = parser.parse_args()

    request = urllib.request.Request(
        args.url,
        headers={"Accept": "application/json", "User-Agent": "virya-signal-release-gate/1"},
    )
    try:
        with urllib.request.urlopen(request, timeout=args.timeout) as response:
            if response.status != 200:
                raise SystemExit(f"APP_LINKS_LIVE=FAIL http={response.status}")
            raw = response.read(MAX_BYTES + 1)
    except urllib.error.HTTPError as error:
        raise SystemExit(f"APP_LINKS_LIVE=FAIL http={error.code}") from error
    except OSError as error:
        raise SystemExit(f"APP_LINKS_LIVE=FAIL network={error}") from error

    if len(raw) > MAX_BYTES:
        raise SystemExit("APP_LINKS_LIVE=FAIL response-too-large")
    try:
        payload = json.loads(raw)
    except json.JSONDecodeError as error:
        raise SystemExit("APP_LINKS_LIVE=FAIL invalid-json") from error
    if not isinstance(payload, list):
        raise SystemExit("APP_LINKS_LIVE=FAIL root-not-array")

    matches = []
    for item in payload:
        if not isinstance(item, dict):
            continue
        target = item.get("target")
        if (
            item.get("relation") == ["delegate_permission/common.handle_all_urls"]
            and isinstance(target, dict)
            and target.get("namespace") == "android_app"
            and target.get("package_name") == args.package
        ):
            matches.append(target)

    if len(matches) != 1:
        raise SystemExit(f"APP_LINKS_LIVE=FAIL delegation-count={len(matches)}")
    fingerprints = matches[0].get("sha256_cert_fingerprints")
    if not isinstance(fingerprints, list) or not fingerprints:
        raise SystemExit("APP_LINKS_LIVE=FAIL fingerprints-missing")
    normalized = [str(value).upper() for value in fingerprints]
    if not all(FINGERPRINT.fullmatch(value) for value in normalized):
        raise SystemExit("APP_LINKS_LIVE=FAIL fingerprint-format")
    if len(set(normalized)) != len(normalized):
        raise SystemExit("APP_LINKS_LIVE=FAIL duplicate-fingerprint")

    print(f"APP_LINKS_LIVE=PASS package={args.package} fingerprints={len(normalized)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
