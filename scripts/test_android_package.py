import importlib.util
import tempfile
import unittest
import zipfile
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
SCRIPT = Path(__file__).with_name("analyze-android-package.py")
SPEC = importlib.util.spec_from_file_location("analyze_android_package", SCRIPT)
MODULE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MODULE)


class AndroidPackageTests(unittest.TestCase):
    def elf(self, alignment: int) -> bytes:
        payload = bytearray(128)
        payload[:6] = b"\x7fELF\x02\x01"
        payload[32:40] = (64).to_bytes(8, "little")
        payload[54:56] = (56).to_bytes(2, "little")
        payload[56:58] = (1).to_bytes(2, "little")
        payload[64:68] = (1).to_bytes(4, "little")
        payload[112:120] = alignment.to_bytes(8, "little")
        return bytes(payload)

    def package(self, root: Path, entries: dict[str, bytes]) -> Path:
        path = root / "virya-signal.apk"
        with zipfile.ZipFile(path, "w", compression=zipfile.ZIP_DEFLATED) as archive:
            for name, payload in entries.items():
                archive.writestr(name, payload)
        return path

    def test_accepts_single_arm64_package(self):
        with tempfile.TemporaryDirectory() as directory:
            path = self.package(
                Path(directory),
                {"lib/arm64-v8a/libvirya.so": self.elf(16384), "assets/index.html": b"html"},
            )
            report = MODULE.analyze(path, "arm64-v8a", 16384)
            self.assertEqual(report["abis"], ["arm64-v8a"])
            self.assertGreater(report["uncompressed_bytes"], 0)
            self.assertEqual(report["categories"]["native"], 128)
            self.assertEqual(report["categories"]["web_assets"], 4)

    def test_rejects_accidental_multi_abi_package(self):
        with tempfile.TemporaryDirectory() as directory:
            path = self.package(
                Path(directory),
                {
                    "lib/arm64-v8a/libvirya.so": self.elf(16384),
                    "lib/x86_64/libvirya.so": self.elf(16384),
                },
            )
            with self.assertRaisesRegex(ValueError, "expected only"):
                MODULE.analyze(path, "arm64-v8a")

    def test_rejects_4k_elf_load_alignment(self):
        with tempfile.TemporaryDirectory() as directory:
            path = self.package(
                Path(directory), {"lib/arm64-v8a/libvirya.so": self.elf(4096)}
            )
            with self.assertRaisesRegex(ValueError, "below 16384"):
                MODULE.analyze(path, "arm64-v8a", 16384)


    def test_signed_android_build_requires_and_verifies_firebase(self):
        workflow = (ROOT / ".github" / "workflows" / "_android-build.yml").read_text()
        self.assertIn("FIREBASE_CONFIG: ${{ secrets.VIRYA_SIGNAL_GOOGLE_SERVICES_JSON_B64 }}", workflow)
        self.assertIn("Missing VIRYA_SIGNAL_GOOGLE_SERVICES_JSON_B64 for signed push-capable build", workflow)
        self.assertIn("--tauri-config src-tauri/tauri.conf.json", workflow)
        self.assertIn("--push-build-config src-tauri/gen/android/push-build-config.json", workflow)
        self.assertIn('check-android-firebase-artifact.py "${package}"', workflow)
        self.assertIn('check-android-app-links-artifact.py "${package}"', workflow)
        self.assertIn('receipt.get("firebaseConfigured") is not True', workflow)
        self.assertIn("SIGNAL_ANDROID_PUSH_BUILD_GATE=PASS", workflow)

    def test_every_google_play_track_requires_push_capable_artifact(self):
        play = (ROOT / ".github/workflows/android-play.yml").read_text()
        verify = play.split("- name: Verify exact build artifact", 1)[1].split("- name: Upload exact AAB", 1)[0]
        self.assertIn(".firebaseConfigured == true", verify)
        self.assertIn(".push.firebaseConfigured == true", verify)
        self.assertNotIn("inputs.play_track == 'production'", verify)

if __name__ == "__main__":
    unittest.main()
