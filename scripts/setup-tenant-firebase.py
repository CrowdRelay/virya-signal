#!/usr/bin/env python3
"""Set up a Firebase project + Android app for a tenant's Signal app.

This automates the one-time Firebase setup that was previously manual:
1. Creates a Firebase project (or reuses an existing one)
2. Registers an Android app with the tenant's package ID
3. Downloads google-services.json
4. Base64-encodes it for GitHub secrets

Requires the Firebase CLI (npm install -g firebase-tools) and authentication
(firebase login).

Usage:
  python3 scripts/setup-tenant-firebase.py \
      --tenant future-metal \
      --package-id music.future-metal.signal \
      --project-id future-metal-signal
"""
from __future__ import annotations

import argparse
import base64
import json
import os
import subprocess
import sys
from pathlib import Path


def run(cmd: list[str], *, capture: bool = False, check: bool = True) -> str:
    print(f"  $ {' '.join(cmd)}", file=sys.stderr)
    result = subprocess.run(
        cmd,
        capture_output=capture,
        text=True,
        check=False,
    )
    if check and result.returncode != 0:
        print(f"ERROR: command failed with exit {result.returncode}", file=sys.stderr)
        if capture:
            print(result.stderr, file=sys.stderr)
        raise SystemExit(1)
    return result.stdout if capture else ""


def firebase_available() -> bool:
    try:
        subprocess.run(["firebase", "--version"], capture_output=True, check=True)
        return True
    except (FileNotFoundError, subprocess.CalledProcessError):
        return False


def project_exists(project_id: str) -> bool:
    result = subprocess.run(
        ["firebase", "projects:list", "--json"],
        capture_output=True,
        text=True,
    )
    if result.returncode != 0:
        return False
    try:
        data = json.loads(result.stdout)
        projects = data.get("result", [])
        return any(p.get("projectId") == project_id for p in projects)
    except (json.JSONDecodeError, KeyError):
        return False


def main() -> None:
    parser = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    parser.add_argument("--tenant", required=True, help="Tenant slug")
    parser.add_argument("--package-id", required=True, help="Android package ID (music.{slug}.signal)")
    parser.add_argument("--project-id", required=True, help="Firebase project ID (e.g. {slug}-signal)")
    parser.add_argument("--output-dir", default="build/tenant-keys", help="Output directory for google-services.json")
    parser.add_argument("--display-name", default="", help="App display name (defaults to '{Tenant} Signal')")
    args = parser.parse_args()

    if not firebase_available():
        print("ERROR: Firebase CLI is not installed.", file=sys.stderr)
        print("Install: npm install -g firebase-tools", file=sys.stderr)
        print("Then: firebase login", file=sys.stderr)
        raise SystemExit(2)

    display_name = args.display_name or f"{args.tenant.title()} Signal"
    output_dir = Path(args.output_dir)
    output_dir.mkdir(parents=True, exist_ok=True)

    # Step 1: Create or reuse the Firebase project
    if project_exists(args.project_id):
        print(f"FIREBASE_SETUP=REUSE project={args.project_id}", file=sys.stderr)
    else:
        print(f"FIREBASE_SETUP=CREATE project={args.project_id}", file=sys.stderr)
        run([
            "firebase", "projects:create", args.project_id,
            "--name", display_name,
        ])

    # Step 2: Add the Android app
    print(f"FIREBASE_SETUP=ADD_APP package={args.package_id}", file=sys.stderr)
    add_app_result = subprocess.run(
        [
            "firebase", "apps:create", "android",
            "--project", args.project_id,
            "--package", args.package_id,
            "--display-name", display_name,
        ],
        capture_output=True,
        text=True,
    )
    # The app may already exist — that's fine
    if add_app_result.returncode != 0:
        if "already" in add_app_result.stderr.lower() or "exists" in add_app_result.stderr.lower():
            print(f"FIREBASE_SETUP=APP_EXISTS package={args.package_id}", file=sys.stderr)
        else:
            print(f"ERROR: firebase apps:create failed: {add_app_result.stderr}", file=sys.stderr)
            raise SystemExit(1)

    # Step 3: Download google-services.json
    print("FIREBASE_SETUP=FETCH_CONFIG", file=sys.stderr)
    config_result = subprocess.run(
        [
            "firebase", "apps:android:download-config",
            "--project", args.project_id,
            "--package", args.package_id,
        ],
        capture_output=True,
        text=True,
    )
    if config_result.returncode != 0:
        # Try fetching by app ID — list apps first
        list_result = subprocess.run(
            ["firebase", "apps:list", "--project", args.project_id, "--json"],
            capture_output=True,
            text=True,
        )
        if list_result.returncode == 0:
            try:
                apps = json.loads(list_result.stdout).get("result", [])
                app_id = next(
                    (a["appId"] for a in apps if args.package_id in a.get("packageName", "")),
                    None,
                )
                if app_id:
                    config_result = subprocess.run(
                        ["firebase", "apps:android:download-config", app_id],
                        capture_output=True,
                        text=True,
                    )
            except (json.JSONDecodeError, KeyError, StopIteration):
                pass

    if config_result.returncode != 0 or not config_result.stdout.strip():
        print(f"ERROR: could not download google-services.json: {config_result.stderr}", file=sys.stderr)
        raise SystemExit(1)

    config_path = output_dir / f"{args.tenant}-google-services.json"
    config_path.write_text(config_result.stdout, encoding="utf-8")
    print(f"FIREBASE_SETUP=CONFIG_WRITTEN path={config_path}", file=sys.stderr)

    # Step 4: Base64-encode for GitHub secrets
    config_b64 = base64.b64encode(config_path.read_bytes()).decode("ascii")
    secrets_path = output_dir / f"{args.tenant}-firebase-secrets.env"
    secret_name = f"TENANT_{args.tenant.upper().replace('-', '_')}_GOOGLE_SERVICES_B64"
    secrets_path.write_text(
        f"{secret_name}={config_b64}\n",
        encoding="utf-8",
    )
    os.chmod(secrets_path, 0o600)

    print("", file=sys.stderr)
    print("FIREBASE_SETUP=SUCCESS", file=sys.stderr)
    print("", file=sys.stderr)
    print(f"google-services.json: {config_path}", file=sys.stderr)
    print(f"Secrets file:          {secrets_path}", file=sys.stderr)
    print("", file=sys.stderr)
    print(f"Add this secret to GitHub:", file=sys.stderr)
    print(f"  {secret_name}", file=sys.stderr)
    print("", file=sys.stderr)
    print("⚠️  Keep google-services.json private — it contains your Firebase API keys.", file=sys.stderr)


if __name__ == "__main__":
    main()
