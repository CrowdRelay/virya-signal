from __future__ import annotations

import base64
import json
import os
import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path


SCRIPT = Path(__file__).with_name("prepare-android.py")
# prepare-android.py derives the Android application ID from tauri.conf.json,
# which is the single source of truth for that permanent Play identity. The
# fixture therefore has to model that file the same way a real checkout does.
APPLICATION_ID = json.loads(
    (Path(__file__).parent.parent / "src-tauri" / "tauri.conf.json").read_text()
)["identifier"]


def seed_tauri_conf(root: Path) -> None:
    (root / "src-tauri").mkdir(parents=True, exist_ok=True)
    (root / "src-tauri" / "tauri.conf.json").write_text(
        json.dumps({"identifier": APPLICATION_ID}), encoding="utf-8"
    )

PUSH_TEMPLATES = SCRIPT.parent.parent / "src-tauri" / "android-push"
ANDROID_ICONS = SCRIPT.parent.parent / "src-tauri" / "icons" / "android"


class PrepareAndroidTests(unittest.TestCase):
    def test_enables_release_shrinking_and_gradle_cache(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            scripts = root / "scripts"
            android = root / "src-tauri" / "gen" / "android"
            app = android / "app"
            scripts.mkdir()
            app.mkdir(parents=True)
            shutil.copy2(SCRIPT, scripts / SCRIPT.name)
            seed_tauri_conf(root)
            push_templates = root / "src-tauri" / "android-push"
            shutil.copytree(PUSH_TEMPLATES, push_templates)
            shutil.copytree(ANDROID_ICONS, root / "src-tauri" / "icons" / "android")

            # Android CI keeps audited launcher resources under src-tauri/icons/android.
            # Model the generated Gradle tree here; prepare-android.py copies the
            # canonical launcher assets into it after `tauri android init`.
            res = app / "src" / "main" / "res"
            adaptive_v26 = res / "mipmap-anydpi-v26"
            adaptive_v26.mkdir(parents=True)
            adaptive_xml = """<?xml version="1.0" encoding="utf-8"?>
<adaptive-icon xmlns:android="http://schemas.android.com/apk/res/android">
    <background android:drawable="@color/ic_launcher_background" />
    <foreground android:drawable="@mipmap/ic_launcher_foreground" />
</adaptive-icon>
"""
            # Current Tauri CLI may emit only the canonical adaptive XML.
            # prepare-android.py must derive the round v33 alias safely.
            (adaptive_v26 / "ic_launcher.xml").write_text(adaptive_xml)
            self.assertFalse((adaptive_v26 / "ic_launcher_round.xml").exists())

            # A generated Gradle fixture may contain only the release build
            # type. The preparer must still configure release shrinking and
            # must not require an explicit debug block.
            gradle = app / "build.gradle.kts"
            gradle.write_text(
                """
plugins {
    id("com.android.application")
}

android {
    compileSdk = 35
    defaultConfig { targetSdk = 35 }
    buildTypes {
        getByName("release") {
            isMinifyEnabled = false
            proguardFiles(getDefaultProguardFile("proguard-android-optimize.txt"), "proguard-rules.pro")
        }
    }
}

dependencies {
    implementation("androidx.core:core-ktx:1.9.0")
}
""".strip()
            )

            manifest = app / "src" / "main" / "AndroidManifest.xml"
            manifest.parent.mkdir(parents=True, exist_ok=True)
            manifest.write_text(
                '<manifest xmlns:android="http://schemas.android.com/apk/res/android"><application android:label="Virya Signal"><activity android:name=".MainActivity" /></application></manifest>'
            )

            result = subprocess.run(
                ["python3", str(scripts / SCRIPT.name)],
                cwd=root,
                text=True,
                capture_output=True,
                check=False,
            )
            self.assertEqual(result.returncode, 0, result.stderr)

            output = gradle.read_text()
            self.assertIn("compileSdk = 36", output)
            self.assertIn("targetSdk = 36", output)
            self.assertIn("isMinifyEnabled = true", output)
            self.assertIn("isShrinkResources = true", output)
            self.assertEqual(output.count("proguardFiles("), 1)

            properties = (android / "gradle.properties").read_text()
            self.assertIn("org.gradle.caching=true", properties)
            self.assertIn("org.gradle.parallel=true", properties)

            for filename in ("ic_launcher.xml", "ic_launcher_round.xml"):
                generated = (res / "mipmap-anydpi-v33" / filename).read_text()
                self.assertEqual(generated.count("<monochrome"), 1)
                self.assertIn('@mipmap/ic_launcher_foreground', generated)
                self.assertIn("<background", generated)
                self.assertIn("<foreground", generated)

            generated_foreground = res / "mipmap-xxxhdpi" / "ic_launcher_foreground.png"
            canonical_foreground = ANDROID_ICONS / "mipmap-xxxhdpi" / "ic_launcher_foreground.png"
            self.assertEqual(generated_foreground.read_bytes(), canonical_foreground.read_bytes())
            self.assertFalse((root / "src-tauri" / "launcher-assets").exists())
            self.assertIn('com.google.firebase:firebase-messaging:25.1.1', output)
            staged = app / "src" / "main" / "java" / "music" / "virya" / "signal" / "push"
            self.assertTrue((staged / "SignalPushPlugin.kt").is_file())
            self.assertTrue((staged / "ViryaFirebaseMessagingService.kt").is_file())
            notification_icon = res / "drawable" / "virya_signal_notification.xml"
            self.assertTrue(notification_icon.is_file())
            self.assertIn('fillColor="#FFFFFFFF"', notification_icon.read_text())
            manifest_text = manifest.read_text()
            self.assertIn("android.permission.POST_NOTIFICATIONS", manifest_text)
            self.assertIn("ViryaFirebaseMessagingService", manifest_text)
            self.assertIn('android:autoVerify="true"', manifest_text)
            self.assertIn('android:host="virya.music"', manifest_text)
            self.assertNotIn('android:host="www.virya.music"', manifest_text)
            self.assertIn('android:pathPrefix="/latarnik"', manifest_text)
            self.assertIn('android:pathPrefix="/pl/latarnik"', manifest_text)
            push_receipt = json.loads((android / "push-build-config.json").read_text())
            self.assertFalse(push_receipt["firebaseConfigured"])
            self.assertEqual(push_receipt["firebaseMessagingVersion"], "25.1.1")
            self.assertFalse(push_receipt["analyticsIncluded"])
            self.assertFalse(push_receipt["crashlyticsIncluded"])
            self.assertFalse((app / "google-services.json").exists())

    def test_configures_firebase_only_with_valid_secret(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            scripts = root / "scripts"
            android = root / "src-tauri" / "gen" / "android"
            app = android / "app"
            scripts.mkdir()
            app.mkdir(parents=True)
            shutil.copy2(SCRIPT, scripts / SCRIPT.name)
            seed_tauri_conf(root)
            shutil.copytree(PUSH_TEMPLATES, root / "src-tauri" / "android-push")
            shutil.copytree(ANDROID_ICONS, root / "src-tauri" / "icons" / "android")
            res = app / "src" / "main" / "res" / "mipmap-anydpi-v26"
            res.mkdir(parents=True)
            res.joinpath("ic_launcher.xml").write_text(
                '<adaptive-icon xmlns:android="http://schemas.android.com/apk/res/android"><background android:drawable="@color/x"/><foreground android:drawable="@mipmap/ic_launcher_foreground"/></adaptive-icon>'
            )
            app.joinpath("build.gradle.kts").write_text(
                'plugins {\n    id("com.android.application")\n}\n\nandroid {\n    compileSdk = 35\n    defaultConfig { targetSdk = 35 }\n    buildTypes {\n        getByName("release") {\n            isMinifyEnabled = false\n        }\n    }\n}\n\ndependencies {\n}\n'
            )
            # Match the real Tauri 2.11 Android template: Groovy settings at
            # the Gradle root, with the Tauri settings script applied below it.
            android.joinpath("settings.gradle").write_text(
                "include ':app'\n\napply from: 'tauri.settings.gradle'\n"
            )
            manifest = app / "src" / "main" / "AndroidManifest.xml"
            manifest.parent.mkdir(parents=True, exist_ok=True)
            manifest.write_text('<manifest xmlns:android="http://schemas.android.com/apk/res/android"><application><activity android:name=".MainActivity" /></application></manifest>')
            document = {
                "project_info": {"project_id": "virya-signal"},
                "client": [{"client_info": {"android_client_info": {"package_name": APPLICATION_ID}}}],
            }
            encoded = base64.b64encode(json.dumps(document).encode()).decode()
            environment = os.environ.copy()
            environment["VIRYA_SIGNAL_GOOGLE_SERVICES_JSON_B64"] = encoded
            result = subprocess.run(
                ["python3", str(scripts / SCRIPT.name)], cwd=root, env=environment, text=True, capture_output=True
            )
            self.assertEqual(result.returncode, 0, result.stderr)
            gradle = app.joinpath("build.gradle.kts").read_text()
            self.assertIn('id("com.google.gms.google-services") version "4.5.0"', gradle)
            settings = android.joinpath("settings.gradle").read_text()
            self.assertTrue(settings.startswith("pluginManagement {"))
            self.assertIn("google()", settings)
            self.assertLess(settings.index("google()"), settings.index("gradlePluginPortal()"))
            self.assertIn("include ':app'", settings)
            self.assertIn("apply from: 'tauri.settings.gradle'", settings)

            # Reused workspaces must not accumulate duplicate Gradle blocks.
            second = subprocess.run(
                ["python3", str(scripts / SCRIPT.name)], cwd=root, env=environment, text=True, capture_output=True
            )
            self.assertEqual(second.returncode, 0, second.stderr)
            settings = android.joinpath("settings.gradle").read_text()
            gradle = app.joinpath("build.gradle.kts").read_text()
            self.assertEqual(settings.count("pluginManagement {"), 1)
            self.assertEqual(settings.count("google()"), 1)
            self.assertEqual(gradle.count('id("com.google.gms.google-services") version "4.5.0"'), 1)

            receipt = json.loads((android / "push-build-config.json").read_text())
            self.assertTrue(receipt["firebaseConfigured"])
            self.assertRegex(receipt["firebaseConfigSha256"], r"^[0-9a-f]{64}$")
            self.assertEqual(json.loads((app / "google-services.json").read_text()), document)


if __name__ == "__main__":
    unittest.main()
