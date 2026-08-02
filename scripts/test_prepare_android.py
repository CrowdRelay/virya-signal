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
            icon_assets = root / "src-tauri" / "launcher-assets" / "android"
            legacy = icon_assets / "mipmap-mdpi"
            legacy.mkdir(parents=True)
            # The preparer only copies bytes; tiny fixtures keep this test free
            # from Pillow/ImageMagick dependencies.
            png = b"\x89PNG\r\n\x1a\nfixture"
            (icon_assets / "ic_launcher_foreground.png").write_bytes(png)
            (legacy / "ic_launcher.png").write_bytes(png)
            (legacy / "ic_launcher_round.png").write_bytes(png)
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
            res = app / "src" / "main" / "res"
            self.assertTrue((res / "drawable" / "ic_launcher_foreground.png").is_file())
            self.assertTrue((res / "mipmap-mdpi" / "ic_launcher.png").is_file())
            self.assertIn(
                "<monochrome",
                (res / "mipmap-anydpi-v33" / "ic_launcher.xml").read_text(),
            )


if __name__ == "__main__":
    unittest.main()
