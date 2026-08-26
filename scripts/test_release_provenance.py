import json
import pathlib
import subprocess
import tempfile
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]
# Single source of truth: the regenerated fan-contract lock. The provenance
# receipt must always carry exactly this digest.
EXPECTED_OPENAPI_SHA = (
    (ROOT / "crates" / "virya-signal-contracts" / "crowdrelay-openapi.sha256")
    .read_text(encoding="ascii")
    .strip()
)

class ReleaseProvenanceTests(unittest.TestCase):
    def test_android_receipt_binds_version_code_firebase_and_crowdrelay_contract(self):
        with tempfile.TemporaryDirectory() as td:
            root = pathlib.Path(td)
            lock = root / "Cargo.lock"; lock.write_text("lock")
            manifest = root / "manifest.json"; manifest.write_text("{}")
            tauri = root / "tauri.json"; tauri.write_text(json.dumps({"version":"0.4.2","bundle":{"android":{"versionCode":2011}}}))
            firebase_sha = "a" * 64
            push = root / "push.json"; push.write_text(json.dumps({"firebaseConfigured":True,"firebaseConfigSha256":firebase_sha,"firebaseMessagingVersion":"25.1.1","googleServicesPluginVersion":"4.5.0"}))
            output = root / "receipt.json"
            subprocess.run([
                "python3", str(ROOT / "scripts/write_release_provenance.py"),
                "--source-sha", "b" * 40,
                "--lockfile", str(lock),
                "--artifact-manifest", str(manifest),
                "--tauri-config", str(tauri),
                "--push-build-config", str(push),
                "--output", str(output),
            ], check=True, stdout=subprocess.PIPE, text=True)
            receipt = json.loads(output.read_text())
            self.assertEqual(receipt["schema"], 2)
            self.assertEqual(receipt["sourceSha"], "b" * 40)
            self.assertEqual(receipt["appVersion"], "0.4.2")
            self.assertEqual(receipt["androidVersionCode"], 2011)
            self.assertTrue(receipt["push"]["firebaseConfigured"])
            self.assertEqual(receipt["push"]["firebaseConfigSha256"], firebase_sha)
            self.assertEqual(receipt["crowdrelayContract"]["apiMajor"], "1")
            self.assertEqual(receipt["crowdrelayContract"]["openapiSha256"], EXPECTED_OPENAPI_SHA)
            self.assertIn("signal_fan_context_v1", receipt["crowdrelayContract"]["requiredCapabilities"])
            self.assertIn("synesthesia_rewards_v1", receipt["crowdrelayContract"]["requiredCapabilities"])

if __name__ == "__main__":
    unittest.main()
