from __future__ import annotations

import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path


SCRIPT = Path(__file__).with_name("prepare-android.py")


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

            # In CI, `cargo tauri icon` runs before prepare-android.py.
            # Model its generated adaptive resources instead of the obsolete
            # hand-copied launcher-assets directory.
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
""".strip()
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

            self.assertFalse((root / "src-tauri" / "launcher-assets").exists())


if __name__ == "__main__":
    unittest.main()
