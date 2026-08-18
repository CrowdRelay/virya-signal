from __future__ import annotations

import hashlib
import importlib.util
import json
import tempfile
import unittest
import zipfile
from pathlib import Path

SCRIPT = Path(__file__).with_name("check-android-firebase-artifact.py")
SPEC = importlib.util.spec_from_file_location("firebase_artifact", SCRIPT)
assert SPEC and SPEC.loader
MODULE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MODULE)


class FirebaseArtifactTests(unittest.TestCase):
    def fixture(self, root: Path, *, include_provider: bool = True, include_app_id: bool = True):
        package = "music.virya.signal"
        app_id = "1:1234567890:android:abcdef123456"
        config = {
            "project_info": {
                "project_number": "1234567890",
                "project_id": "virya-signal",
            },
            "client": [
                {
                    "client_info": {
                        "mobilesdk_app_id": app_id,
                        "android_client_info": {"package_name": package},
                    }
                }
            ],
        }
        config_path = root / "google-services.json"
        config_path.write_text(json.dumps(config))
        receipt_path = root / "push-build-config.json"
        receipt_path.write_text(
            json.dumps(
                {
                    "firebaseConfigured": True,
                    "firebaseConfigSha256": hashlib.sha256(config_path.read_bytes()).hexdigest(),
                }
            )
        )
        tauri_path = root / "tauri.conf.json"
        tauri_path.write_text(json.dumps({"identifier": package}))
        aab = root / "app.aab"
        resources = (
            b"virya-signal\x00"
            + b"1234567890\x00"
            + (app_id.encode() if include_app_id else b"missing-app-id")
        )
        manifest = b"ViryaFirebaseMessagingService\x00com.google.firebase.MESSAGING_EVENT"
        if include_provider:
            manifest += b"\x00com.google.firebase.provider.FirebaseInitProvider"
        dex = b"dex\n035\x00Lcom/google/firebase/FirebaseApp;\x00FirebaseMessaging"
        with zipfile.ZipFile(aab, "w") as archive:
            archive.writestr("base/resources.pb", resources)
            archive.writestr("base/manifest/AndroidManifest.xml", manifest)
            archive.writestr("base/dex/classes.dex", dex)
        return aab, config_path, receipt_path, tauri_path

    def test_accepts_firebase_wired_aab(self):
        with tempfile.TemporaryDirectory() as directory:
            paths = self.fixture(Path(directory))
            report = MODULE.verify(
                paths[0], config_path=paths[1], receipt_path=paths[2], tauri_config_path=paths[3]
            )
            self.assertEqual(report["package"], "music.virya.signal")
            self.assertEqual(report["project"], "virya-signal")

    def test_rejects_source_config_that_never_reached_resources(self):
        with tempfile.TemporaryDirectory() as directory:
            paths = self.fixture(Path(directory), include_app_id=False)
            with self.assertRaisesRegex(ValueError, "compiled google_app_id"):
                MODULE.verify(
                    paths[0], config_path=paths[1], receipt_path=paths[2], tauri_config_path=paths[3]
                )

    def test_rejects_missing_firebase_auto_init_provider(self):
        with tempfile.TemporaryDirectory() as directory:
            paths = self.fixture(Path(directory), include_provider=False)
            with self.assertRaisesRegex(ValueError, "FirebaseInitProvider"):
                MODULE.verify(
                    paths[0], config_path=paths[1], receipt_path=paths[2], tauri_config_path=paths[3]
                )

    def test_rejects_receipt_config_drift(self):
        with tempfile.TemporaryDirectory() as directory:
            paths = self.fixture(Path(directory))
            paths[1].write_text(paths[1].read_text() + "\n")
            with self.assertRaisesRegex(ValueError, "changed after Android preparation"):
                MODULE.verify(
                    paths[0], config_path=paths[1], receipt_path=paths[2], tauri_config_path=paths[3]
                )


if __name__ == "__main__":
    unittest.main()
