#!/usr/bin/env python3
"""Write a secretless content-root receipt for an immutable release artifact."""
from __future__ import annotations
import argparse, hashlib, json, re
from pathlib import Path

GIT_SHA = re.compile(r"^[0-9a-f]{40}$")
OPENAPI_SHA = re.compile(r"^[0-9a-f]{64}$")
ROOT = Path(__file__).resolve().parents[1]
CONTRACT_SOURCE = ROOT / "crates/virya-signal-contracts/src/fan_wire.generated.rs"
CONTRACT_RE = re.compile(r"@crowdrelay-openapi-sha256\s+([0-9a-f]{64})")
REQUIRED_CAPABILITIES = [
    "signal_fan_context_v1",
    "synesthesia_rewards_v1",
    "synesthesia_leaderboard_v1",
    "ticketing_v1",
]

def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()

def crowdrelay_contract() -> dict[str, object]:
    if not CONTRACT_SOURCE.is_file():
        raise SystemExit(f"CrowdRelay generated contract source missing: {CONTRACT_SOURCE}")
    match = CONTRACT_RE.search(CONTRACT_SOURCE.read_text())
    if match is None or OPENAPI_SHA.fullmatch(match.group(1)) is None:
        raise SystemExit("CrowdRelay OpenAPI fingerprint missing from generated Signal contract")
    return {
        "apiMajor": "1",
        "openapiSha256": match.group(1),
        "requiredCapabilities": REQUIRED_CAPABILITIES,
    }

def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--source-sha", required=True)
    parser.add_argument("--lockfile", required=True, type=Path)
    parser.add_argument("--artifact-manifest", required=True, type=Path)
    parser.add_argument("--tauri-config", type=Path)
    parser.add_argument("--push-build-config", type=Path)
    parser.add_argument("--output", required=True, type=Path)
    args = parser.parse_args()
    if not GIT_SHA.fullmatch(args.source_sha):
        raise SystemExit("source SHA must be a full lowercase Git SHA")
    for path in (args.lockfile, args.artifact_manifest):
        if not path.is_file():
            raise SystemExit(f"required provenance input missing: {path}")
    receipt = {
        "schema": 2,
        "sourceSha": args.source_sha,
        "dependencyLockSha256": sha256(args.lockfile),
        "artifactManifestSha256": sha256(args.artifact_manifest),
        "crowdrelayContract": crowdrelay_contract(),
    }
    if args.tauri_config is not None:
        if not args.tauri_config.is_file():
            raise SystemExit(f"Tauri config missing: {args.tauri_config}")
        config = json.loads(args.tauri_config.read_text())
        receipt["appVersion"] = str(config.get("version", ""))
        receipt["androidVersionCode"] = int(config["bundle"]["android"]["versionCode"])
    if args.push_build_config is not None:
        if not args.push_build_config.is_file():
            raise SystemExit(f"push build config missing: {args.push_build_config}")
        push = json.loads(args.push_build_config.read_text())
        firebase_sha = push.get("firebaseConfigSha256")
        if firebase_sha is not None and not re.fullmatch(r"[0-9a-f]{64}", str(firebase_sha)):
            raise SystemExit("invalid Firebase config SHA-256 in push build receipt")
        receipt["push"] = {
            "firebaseConfigured": bool(push.get("firebaseConfigured")),
            "firebaseConfigSha256": firebase_sha,
            "firebaseMessagingVersion": push.get("firebaseMessagingVersion"),
            "googleServicesPluginVersion": push.get("googleServicesPluginVersion"),
        }
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(receipt, indent=2, sort_keys=True) + "\n")
    print(
        "RELEASE_PROVENANCE=PASS "
        f"source={receipt['sourceSha']} manifest={receipt['artifactManifestSha256']} "
        f"crowdrelay={receipt['crowdrelayContract']['openapiSha256']} "
        f"version={receipt.get('appVersion', 'n/a')} code={receipt.get('androidVersionCode', 'n/a')} "
        f"firebase={receipt.get('push', {}).get('firebaseConfigured', 'n/a')}"
    )
    return 0

if __name__ == "__main__":
    raise SystemExit(main())
