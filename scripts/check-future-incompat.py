#!/usr/bin/env python3
from __future__ import annotations
import json, os, subprocess
try:
    import tomllib
except ModuleNotFoundError:
    tomllib = None
from datetime import date, datetime
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
POLICY = json.loads((ROOT / "security/future-incompat-budget.json").read_text())
def _locked_packages_and_parents(name: str, version: str):
    force_metadata = os.environ.get("FUTURE_INCOMPAT_FORCE_CARGO_METADATA") == "1"
    if tomllib is not None and not force_metadata:
        lock = tomllib.loads((ROOT / "Cargo.lock").read_text())
        packages = lock.get("package", [])
        matches = [p for p in packages if p.get("name") == name and p.get("version") == version]
        parents = sorted(
            f"{p['name']}@{p['version']}"
            for p in packages
            if any(str(dep).split()[0] == name for dep in p.get("dependencies", []))
        )
        return len(matches), parents

    try:
        metadata = json.loads(
            subprocess.check_output(
                ["cargo", "metadata", "--locked", "--offline", "--format-version", "1"],
                cwd=ROOT,
                text=True,
            )
        )
    except (OSError, subprocess.CalledProcessError, json.JSONDecodeError) as exc:
        raise SystemExit(f"FUTURE_INCOMPAT=FAIL cargo metadata fallback failed: {exc}") from exc

    package_by_id = {
        package["id"]: package
        for package in metadata.get("packages", [])
        if "id" in package
    }
    target_ids = {
        package["id"]
        for package in metadata.get("packages", [])
        if package.get("name") == name and package.get("version") == version
    }
    parents = sorted(
        {
            f"{package_by_id[node['id']]['name']}@{package_by_id[node['id']]['version']}"
            for node in metadata.get("resolve", {}).get("nodes", [])
            if node.get("id") in package_by_id
            and any(dep.get("pkg") in target_ids for dep in node.get("deps", []))
        }
    )
    return len(target_ids), parents


name = POLICY["package"]
version = POLICY["version"]
match_count, parents = _locked_packages_and_parents(name, version)
if match_count != 1:
    raise SystemExit(f"FUTURE_INCOMPAT=FAIL budget stale: expected exactly {name}@{version}, found {match_count}")
expected = sorted(POLICY.get("allowedParents", []))
if parents != expected:
    raise SystemExit(f"FUTURE_INCOMPAT=FAIL parent graph drift expected={expected} actual={parents}")
now_text = os.environ.get("FUTURE_COMPAT_NOW", "")
now = datetime.fromisoformat(now_text).date() if now_text else date.today()
review = date.fromisoformat(POLICY["reviewBy"])
remaining = (review - now).days
fail_before = int(POLICY.get("failBeforeDays", 21))
if remaining <= fail_before:
    raise SystemExit(
        f"FUTURE_INCOMPAT=FAIL {name}@{version} review window reached; "
        f"days_remaining={remaining} review_by={review.isoformat()}"
    )
print(
    f"FUTURE_INCOMPAT=PASS package={name}@{version} parents={len(parents)} "
    f"days_remaining={remaining}"
)
