#!/usr/bin/env python3
"""Generate a private, short-lived Virya Signal setup envelope and optional QR."""
from __future__ import annotations

import argparse
import base64
import json
import os
import shutil
import subprocess
import time
from pathlib import Path


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--token", default=os.getenv("CROWDRELAY_OPERATOR_TOKEN"))
    parser.add_argument("--role", choices=("owner", "staff"), default="staff")
    parser.add_argument("--name", default="Virya staff")
    parser.add_argument("--api", default="https://signal-api.virya.music/v1/")
    parser.add_argument("--ttl-minutes", type=int, default=10)
    parser.add_argument("--svg", type=Path)
    args = parser.parse_args()

    token = (args.token or "").strip()
    if len(token) < 24:
        raise SystemExit("Pass --token or CROWDRELAY_OPERATOR_TOKEN")
    if not 1 <= args.ttl_minutes <= 30:
        raise SystemExit("TTL must be between 1 and 30 minutes")

    payload = {
        "version": 1,
        "apiBaseUrl": args.api,
        "displayName": args.name.strip(),
        "role": args.role,
        "bearerToken": token,
        "expiresAt": int(time.time()) + args.ttl_minutes * 60,
    }
    encoded = base64.urlsafe_b64encode(
        json.dumps(payload, separators=(",", ":"), ensure_ascii=False).encode()
    ).decode().rstrip("=")
    uri = f"virya-signal://pair?payload={encoded}"
    print(uri)

    if args.svg:
        qrencode = shutil.which("qrencode")
        if not qrencode:
            raise SystemExit("Install qrencode to use --svg")
        subprocess.run([qrencode, "-t", "SVG", "-o", str(args.svg), uri], check=True)
        os.chmod(args.svg, 0o600)


if __name__ == "__main__":
    main()
