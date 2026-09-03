#!/usr/bin/env python3
"""Fetch a tenant's mobile app configuration from the CrowdRelay Control Plane.

Outputs a JSON object with everything the per-tenant build script needs:
  - slug, display_name
  - package_id (music.{slug}.signal)
  - app_name ({display_name} Signal)
  - branding_palette (10-color palette or null)
  - signal_base_url (the tenant's public Signal site URL)
  - crowdrelay_base_url (the tenant's CrowdRelay API URL)
  - play_store_url (the tenant's Signal Play Store URL or null)

Usage:
  python3 scripts/fetch-tenant-config.py \
      --tenant virya \
      --control-plane-url https://control.virya.music \
      --token $CONTROL_PLANE_ADMIN_TOKEN

The token must be a platform admin session token. The script uses the same
/api/v1 prefix as the SPA.
"""
from __future__ import annotations

import argparse
import json
import os
import sys
import urllib.error
import urllib.request
from typing import Any


def fetch_tenant(base_url: str, slug: str, token: str) -> dict[str, Any]:
    url = f"{base_url.rstrip('/')}/api/v1/tenants/{slug}"
    request = urllib.request.Request(
        url,
        headers={
            "Authorization": f"Bearer {token}",
            "Accept": "application/json",
        },
    )
    try:
        with urllib.request.urlopen(request, timeout=15.0) as response:
            return json.loads(response.read())
    except urllib.error.HTTPError as exc:
        body = exc.read().decode("utf-8", errors="replace") if exc.fp else ""
        print(f"ERROR: control plane returned {exc.code}: {body}", file=sys.stderr)
        raise SystemExit(1) from exc
    except urllib.error.URLError as exc:
        print(f"ERROR: could not reach control plane at {url}: {exc}", file=sys.stderr)
        raise SystemExit(1) from exc


def derive_config(tenant: dict[str, Any]) -> dict[str, Any]:
    slug = tenant["slug"]
    display_name = tenant["displayName"]
    package_id = f"music.{slug}.signal"
    app_name = f"{display_name} Signal"
    palette = tenant.get("brandingPalette")
    signal_base_url = tenant.get("signalBaseUrl")
    crowdrelay_base_url = tenant.get("crowdrelayBaseUrl")
    play_store_url = tenant.get("signalPlayStoreUrl")

    if not signal_base_url:
        print(
            f"WARNING: tenant {slug} has no signalBaseUrl; the app will use the default API host",
            file=sys.stderr,
        )

    return {
        "slug": slug,
        "displayName": display_name,
        "packageId": package_id,
        "appName": app_name,
        "brandingPalette": palette,
        "signalBaseUrl": signal_base_url,
        "crowdrelayBaseUrl": crowdrelay_base_url,
        "playStoreUrl": play_store_url,
    }


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--tenant", required=True, help="Tenant slug")
    parser.add_argument("--control-plane-url", required=True, help="Control Plane API base URL")
    parser.add_argument("--token", default=os.environ.get("CONTROL_PLANE_ADMIN_TOKEN", ""), help="Platform admin token")
    parser.add_argument("--output", default="-", help="Output path (- for stdout)")
    args = parser.parse_args()

    if not args.token:
        print("ERROR: --token or CONTROL_PLANE_ADMIN_TOKEN env var is required", file=sys.stderr)
        raise SystemExit(2)

    tenant = fetch_tenant(args.control_plane_url, args.tenant, args.token)
    config = derive_config(tenant)

    output = json.dumps(config, indent=2, separators=(",", ": "))
    if args.output == "-":
        print(output)
    else:
        Path(args.output).write_text(output + "\n", encoding="utf-8")
        print(f"Wrote tenant config to {args.output}", file=sys.stderr)


if __name__ == "__main__":
    from pathlib import Path  # noqa: E402

    main()
