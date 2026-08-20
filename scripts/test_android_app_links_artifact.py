from __future__ import annotations

import importlib.util
import json
import tempfile
import unittest
import zipfile
from pathlib import Path

SCRIPT = Path(__file__).with_name("check-android-app-links-artifact.py")
SPEC = importlib.util.spec_from_file_location("app_links_artifact", SCRIPT)
assert SPEC and SPEC.loader
MODULE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MODULE)


class AppLinksArtifactTests(unittest.TestCase):
    def fixture(self, root: Path, *, paths=MODULE.REQUIRED_PATHS, auto_verify=True):
        package = "music.virya.signal"
        tauri = root / "tauri.conf.json"
        tauri.write_text(json.dumps({"identifier": package}))
        manifest = b"\x00".join(
            value.encode()
            for value in [
                package,
                "android.intent.action.VIEW",
                "android.intent.category.DEFAULT",
                "android.intent.category.BROWSABLE",
                *( ["autoVerify"] if auto_verify else [] ),
                "https",
                "virya.music",
                "virya-signal",
                "my-signal",
                *paths,
            ]
        )
        aab = root / "app.aab"
        with zipfile.ZipFile(aab, "w") as archive:
            archive.writestr("base/manifest/AndroidManifest.xml", manifest)
        return aab, tauri

    def test_accepts_all_signal_and_synesthesia_links(self):
        with tempfile.TemporaryDirectory() as directory:
            aab, tauri = self.fixture(Path(directory))
            report = MODULE.verify(aab, tauri_config_path=tauri)
            self.assertEqual(report["host"], "virya.music")
            self.assertEqual(len(report["paths"]), 4)

    def test_rejects_old_latarnik_only_manifest(self):
        with tempfile.TemporaryDirectory() as directory:
            aab, tauri = self.fixture(Path(directory), paths=("/latarnik", "/pl/latarnik"))
            with self.assertRaisesRegex(ValueError, "/my-signal"):
                MODULE.verify(aab, tauri_config_path=tauri)

    def test_rejects_non_verified_filter(self):
        with tempfile.TemporaryDirectory() as directory:
            aab, tauri = self.fixture(Path(directory), auto_verify=False)
            with self.assertRaisesRegex(ValueError, "autoVerify"):
                MODULE.verify(aab, tauri_config_path=tauri)


if __name__ == "__main__":
    unittest.main()
