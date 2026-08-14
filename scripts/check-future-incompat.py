#!/usr/bin/env python3
from __future__ import annotations
import json, os, tomllib
from datetime import date, datetime
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
POLICY = json.loads((ROOT / "security/future-incompat-budget.json").read_text())
LOCK = tomllib.loads((ROOT / "Cargo.lock").read_text())

packages = LOCK.get("package", [])
name = POLICY["package"]
version = POLICY["version"]
matches = [p for p in packages if p.get("name") == name and p.get("version") == version]
if len(matches) != 1:
    raise SystemExit(f"FUTURE_INCOMPAT=FAIL budget stale: expected exactly {name}@{version}, found {len(matches)}")
parents = sorted(
    f"{p['name']}@{p['version']}"
    for p in packages
    if any(str(dep).split()[0] == name for dep in p.get("dependencies", []))
)
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
