#!/usr/bin/env python3
from __future__ import annotations

import argparse
import hashlib
import json
import re
import zipfile
from pathlib import Path

SHA256_RE = re.compile(r"^[0-9a-f]{64}$")


def load_json(path: Path) -> dict:
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        raise ValueError(f"could not read JSON {path}: {error}") from error


def expected_firebase_identity(config: dict, package_name: str) -> tuple[str, str, str]:
    project = config.get("project_info") or {}
    project_id = str(project.get("project_id") or "").strip()
    project_number = str(project.get("project_number") or "").strip()
    if not project_id or not project_number:
        raise ValueError("google-services.json is missing project_id/project_number")

    matches: list[str] = []
    for client in config.get("client") or []:
        info = client.get("client_info") or {}
        android = info.get("android_client_info") or {}
        if android.get("package_name") == package_name:
            app_id = str(info.get("mobilesdk_app_id") or "").strip()
            if app_id:
                matches.append(app_id)
    if len(matches) != 1:
        raise ValueError(
            f"google-services.json must contain exactly one Firebase app id for {package_name}"
        )
    return project_id, project_number, matches[0]


def require_bytes(haystack: bytes, needle: str, label: str) -> None:
    if needle.encode("utf-8") not in haystack:
        raise ValueError(f"AAB is missing {label}")


def verify(
    aab: Path,
    *,
    config_path: Path,
    receipt_path: Path,
    tauri_config_path: Path,
) -> dict[str, str]:
    if not aab.is_file():
        raise ValueError(f"AAB does not exist: {aab}")
    if not config_path.is_file():
        raise ValueError(f"Firebase config does not exist: {config_path}")

    receipt = load_json(receipt_path)
    if receipt.get("firebaseConfigured") is not True:
        raise ValueError("push build receipt says Firebase is not configured")
    receipt_sha = str(receipt.get("firebaseConfigSha256") or "")
    if not SHA256_RE.fullmatch(receipt_sha):
        raise ValueError("push build receipt has an invalid Firebase config SHA-256")
    actual_sha = hashlib.sha256(config_path.read_bytes()).hexdigest()
    if actual_sha != receipt_sha:
        raise ValueError("Firebase config changed after Android preparation")

    tauri = load_json(tauri_config_path)
    package_name = str(tauri.get("identifier") or "").strip()
    if not package_name:
        raise ValueError("tauri.conf.json is missing identifier")
    project_id, project_number, app_id = expected_firebase_identity(
        load_json(config_path), package_name
    )

    try:
        with zipfile.ZipFile(aab) as archive:
            corrupt = archive.testzip()
            if corrupt:
                raise ValueError(f"AAB contains a corrupt entry: {corrupt}")
            names = set(archive.namelist())
            resources_name = "base/resources.pb"
            manifest_name = "base/manifest/AndroidManifest.xml"
            if resources_name not in names:
                raise ValueError(f"AAB is missing {resources_name}")
            if manifest_name not in names:
                raise ValueError(f"AAB is missing {manifest_name}")
            resources = archive.read(resources_name)
            manifest = archive.read(manifest_name)
            dex = b"".join(
                archive.read(name)
                for name in sorted(names)
                if name.startswith("base/dex/") and name.endswith(".dex")
            )
    except zipfile.BadZipFile as error:
        raise ValueError("AAB is not a valid ZIP container") from error

    # google-services Gradle plugin turns the selected client into Android
    # resources. Checking the compiled protobuf catches the exact failure mode
    # where the source JSON/receipt exists but the plugin did not affect the AAB.
    require_bytes(resources, app_id, "compiled google_app_id Firebase resource")
    require_bytes(resources, project_id, "compiled Firebase project_id resource")
    require_bytes(resources, project_number, "compiled Firebase sender/project number resource")

    # FirebaseApp auto-init depends on this manifest provider. The custom
    # messaging service proves our app-side FCM hook survived manifest merging.
    require_bytes(
        manifest,
        "com.google.firebase.provider.FirebaseInitProvider",
        "FirebaseInitProvider in merged manifest",
    )
    require_bytes(
        manifest,
        "ViryaFirebaseMessagingService",
        "ViryaFirebaseMessagingService in merged manifest",
    )
    require_bytes(
        manifest,
        "com.google.firebase.MESSAGING_EVENT",
        "Firebase messaging intent action in merged manifest",
    )

    # A signed build with resources but without the Firebase runtime would still
    # fail at startup. R8 is disabled in the safe release profile, so these class
    # descriptors must remain visible in the final DEX payload.
    if not dex:
        raise ValueError("AAB has no base DEX payload")
    if b"FirebaseApp" not in dex:
        raise ValueError("AAB DEX payload is missing FirebaseApp runtime")
    if b"FirebaseMessaging" not in dex:
        raise ValueError("AAB DEX payload is missing FirebaseMessaging runtime")

    return {
        "package": package_name,
        "project": project_id,
        "firebase_config_sha256": actual_sha,
    }


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Verify that the final Play AAB really contains working Firebase wiring"
    )
    parser.add_argument("aab", type=Path)
    parser.add_argument(
        "--config",
        type=Path,
        default=Path("src-tauri/gen/android/app/google-services.json"),
    )
    parser.add_argument(
        "--receipt",
        type=Path,
        default=Path("src-tauri/gen/android/push-build-config.json"),
    )
    parser.add_argument(
        "--tauri-config",
        type=Path,
        default=Path("src-tauri/tauri.conf.json"),
    )
    args = parser.parse_args()
    try:
        report = verify(
            args.aab,
            config_path=args.config,
            receipt_path=args.receipt,
            tauri_config_path=args.tauri_config,
        )
    except ValueError as error:
        parser.error(str(error))
    print(
        "SIGNAL_AAB_FIREBASE_GATE=PASS "
        f"package={report['package']} project={report['project']} "
        f"config_sha256={report['firebase_config_sha256']}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
