#!/usr/bin/env python3
"""Single source of truth for the Android application ID.

Google Play treats the application ID as permanent identity from the first
upload onwards: it cannot be changed later without shipping a different app and
losing every install, rating and subscription. The ID was previously repeated as
a literal in both Play publishing workflows, three helper scripts, the E2E
driver and the google-services validation, which is exactly the shape that
drifts silently and publishes under the wrong identity.

`src-tauri/tauri.conf.json` is authoritative because it is what actually
generates the Android project. Everything else must agree with it.
"""
from __future__ import annotations

import json
import pathlib
import re
import sys
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]
APPLICATION_ID = json.loads(
    (ROOT / "src-tauri/tauri.conf.json").read_text(encoding="utf-8")
)["identifier"]
LEGACY_IDS = ("music.virya.control",)


def read(relative: str) -> str:
    return (ROOT / relative).read_text(encoding="utf-8")


class AndroidApplicationIdContract(unittest.TestCase):
    def test_application_id_is_a_valid_permanent_play_identity(self):
        self.assertRegex(APPLICATION_ID, r"^[a-z][a-z0-9_]*(\.[a-z][a-z0-9_]*)+$")
        # Play rejects some reserved segments outright.
        for reserved in (".example.", ".test."):
            self.assertNotIn(reserved, f".{APPLICATION_ID}.")

    def test_play_publishing_workflows_upload_under_the_configured_id(self):
        for workflow in (".github/workflows/mobile-release.yml", ".github/workflows/android-play.yml"):
            source = read(workflow)
            self.assertIn(
                f"packageName: {APPLICATION_ID}",
                source,
                f"{workflow} must publish the application ID from tauri.conf.json",
            )

    def test_no_source_still_references_a_retired_application_id(self):
        tracked = [
            "README.md",
            "src-tauri/tauri.conf.json",
            "scripts/prepare-android.py",
            "scripts/capture-android-crash.fish",
            "scripts/profile-android.sh",
            "scripts/e2e/android_journeys.py",
            "scripts/test_prepare_android.py",
            ".github/workflows/mobile-release.yml",
            ".github/workflows/android-play.yml",
            ".github/workflows/android-e2e.yml",
        ]
        for relative in tracked:
            source = read(relative)
            for legacy in LEGACY_IDS:
                self.assertNotIn(
                    legacy,
                    source,
                    f"{relative} still references the retired application ID {legacy}",
                )

    def test_helper_scripts_target_the_configured_id(self):
        for relative in (
            "scripts/capture-android-crash.fish",
            "scripts/profile-android.sh",
            "scripts/e2e/android_journeys.py",
            ".github/workflows/android-e2e.yml",
        ):
            self.assertIn(
                APPLICATION_ID,
                read(relative),
                f"{relative} must target the configured application ID",
            )

    def test_prepare_android_derives_the_id_instead_of_repeating_it(self):
        source = read("scripts/prepare-android.py")
        self.assertIn("APPLICATION_ID = _application_id()", source)
        self.assertIn("tauri.conf.json", source)
        # The literal must not be reintroduced alongside the derived value.
        self.assertNotIn(f'"{APPLICATION_ID}"', source)

    def test_push_transport_stays_a_subpackage_of_the_application_id(self):
        source = read("scripts/prepare-android.py")
        self.assertIn('PUSH_PACKAGE = f"{APPLICATION_ID}.push"', source)
        expected = f"{APPLICATION_ID}.push"
        # The Kotlin sources and the Rust plugin identifier are compiled against
        # that package name, so they must agree with the derived value.
        self.assertIn(f"package {expected}", read("src-tauri/android-push/SignalPushPlugin.kt"))
        self.assertIn(
            f"package {expected}",
            read("src-tauri/android-push/ViryaFirebaseMessagingService.kt"),
        )
        self.assertIn(f'"{expected}"', read("src-tauri/src/push_plugin.rs"))

    def test_google_services_validation_is_bound_to_the_configured_id(self):
        source = read("scripts/prepare-android.py")
        self.assertIn("APPLICATION_ID not in packages", source)
        # A stale Firebase config for a retired ID must fail loudly, because FCM
        # silently does nothing when google-services.json targets another package.
        self.assertIn("google-services.json does not target", source)


if __name__ == "__main__":
    unittest.main()
