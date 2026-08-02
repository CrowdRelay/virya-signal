#!/usr/bin/env python3
from __future__ import annotations

import argparse
import re
import shutil
from pathlib import Path

parser = argparse.ArgumentParser()
parser.add_argument("--signing", action="store_true")
args = parser.parse_args()

root = Path(__file__).resolve().parents[1]
android = root / "src-tauri" / "gen" / "android"
gradle = android / "app" / "build.gradle.kts"
if not gradle.is_file():
    raise SystemExit(f"missing generated Android project: {gradle}")

text = gradle.read_text(encoding="utf-8")
text, compile_count = re.subn(r"\bcompileSdk\s*=\s*\d+", "compileSdk = 36", text)
text, target_count = re.subn(r"\btargetSdk\s*=\s*\d+", "targetSdk = 36", text)
if compile_count != 1 or target_count != 1:
    raise SystemExit(
        "expected exactly one compileSdk and targetSdk in generated Gradle file "
        f"(found {compile_count}/{target_count})"
    )

# Release APK/AAB builds should ship neither unused CameraX/ML Kit bytecode nor
# dead Android resources. This is intentionally applied to the generated app
# instead of checking generated files into source control.
text, minify_count = re.subn(
    r"isMinifyEnabled\s*=\s*false", "isMinifyEnabled = true", text, count=1
)
text, shrink_count = re.subn(
    r"isShrinkResources\s*=\s*false", "isShrinkResources = true", text, count=1
)
release_patterns = [r'(getByName\("release"\)\s*\{)', r'(named\("release"\)\s*\{)']
if minify_count == 0 and "isMinifyEnabled = true" not in text:
    for pattern in release_patterns:
        text, count = re.subn(pattern, r'\1\n            isMinifyEnabled = true', text, count=1)
        if count:
            break
    else:
        raise SystemExit("could not locate release build type for R8 configuration")
if shrink_count == 0 and "isShrinkResources = true" not in text:
    text = text.replace(
        "isMinifyEnabled = true",
        "isMinifyEnabled = true\n            isShrinkResources = true",
        1,
    )
if "proguardFiles(" not in text:
    text = text.replace(
        "isShrinkResources = true",
        'isShrinkResources = true\n            proguardFiles(getDefaultProguardFile("proguard-android-optimize.txt"), "proguard-rules.pro")',
        1,
    )

if args.signing:
    properties = android / "keystore.properties"
    if not properties.is_file():
        raise SystemExit("keystore.properties is required for a signed release")
    imports = []
    if "import java.io.FileInputStream" not in text:
        imports.append("import java.io.FileInputStream")
    if "import java.util.Properties" not in text:
        imports.append("import java.util.Properties")
    if imports:
        text = "\n".join(imports) + "\n" + text

    if 'create("release")' not in text:
        signing = '''
    signingConfigs {
        create("release") {
            val keystorePropertiesFile = rootProject.file("keystore.properties")
            val keystoreProperties = Properties()
            keystoreProperties.load(FileInputStream(keystorePropertiesFile))
            keyAlias = keystoreProperties["keyAlias"] as String
            keyPassword = keystoreProperties["password"] as String
            storeFile = file(keystoreProperties["storeFile"] as String)
            storePassword = keystoreProperties["password"] as String
        }
    }

'''
        marker = "    buildTypes {"
        if marker not in text:
            raise SystemExit("could not locate buildTypes in generated Gradle file")
        text = text.replace(marker, signing + marker, 1)

    if "signingConfig = signingConfigs.getByName(\"release\")" not in text:
        patterns = [
            r'(getByName\("release"\)\s*\{)',
            r'(named\("release"\)\s*\{)',
        ]
        for pattern in patterns:
            text, count = re.subn(
                pattern,
                r'\1\n            signingConfig = signingConfigs.getByName("release")',
                text,
                count=1,
            )
            if count:
                break
        else:
            raise SystemExit("could not locate release build type in generated Gradle file")

gradle.write_text(text, encoding="utf-8")

# Install deterministic professional launcher assets after Tauri generates the
# Android project. This keeps source control free of the generated Gradle tree
# while still shipping adaptive, round and legacy icons on every CI build.
icon_assets = root / "src-tauri" / "launcher-assets" / "android"
res = android / "app" / "src" / "main" / "res"
if not icon_assets.is_dir():
    raise SystemExit(f"missing Android icon assets: {icon_assets}")

for source_dir in sorted(icon_assets.glob("mipmap-*")):
    if not source_dir.is_dir():
        continue
    destination = res / source_dir.name
    destination.mkdir(parents=True, exist_ok=True)
    for source in source_dir.glob("*.png"):
        shutil.copy2(source, destination / source.name)

drawable = res / "drawable"
drawable.mkdir(parents=True, exist_ok=True)
shutil.copy2(icon_assets / "ic_launcher_foreground.png", drawable / "ic_launcher_foreground.png")
(drawable / "ic_launcher_background.xml").write_text(
    """<?xml version=\"1.0\" encoding=\"utf-8\"?>
<shape xmlns:android=\"http://schemas.android.com/apk/res/android\" android:shape=\"rectangle\">
    <solid android:color=\"#080808\" />
</shape>
""",
    encoding="utf-8",
)

adaptive = """<?xml version=\"1.0\" encoding=\"utf-8\"?>
<adaptive-icon xmlns:android=\"http://schemas.android.com/apk/res/android\">
    <background android:drawable=\"@drawable/ic_launcher_background\" />
    <foreground android:drawable=\"@drawable/ic_launcher_foreground\" />
</adaptive-icon>
"""
for qualifier in ("mipmap-anydpi-v26",):
    destination = res / qualifier
    destination.mkdir(parents=True, exist_ok=True)
    (destination / "ic_launcher.xml").write_text(adaptive, encoding="utf-8")
    (destination / "ic_launcher_round.xml").write_text(adaptive, encoding="utf-8")

themed = """<?xml version=\"1.0\" encoding=\"utf-8\"?>
<adaptive-icon xmlns:android=\"http://schemas.android.com/apk/res/android\">
    <background android:drawable=\"@drawable/ic_launcher_background\" />
    <foreground android:drawable=\"@drawable/ic_launcher_foreground\" />
    <monochrome android:drawable=\"@drawable/ic_launcher_foreground\" />
</adaptive-icon>
"""
destination = res / "mipmap-anydpi-v33"
destination.mkdir(parents=True, exist_ok=True)
(destination / "ic_launcher.xml").write_text(themed, encoding="utf-8")
(destination / "ic_launcher_round.xml").write_text(themed, encoding="utf-8")

gradle_properties = android / "gradle.properties"
properties_text = gradle_properties.read_text(encoding="utf-8") if gradle_properties.exists() else ""
managed_properties = {
    "org.gradle.caching": "true",
    "org.gradle.parallel": "true",
    "org.gradle.jvmargs": "-Xmx3g -XX:MaxMetaspaceSize=1g -Dfile.encoding=UTF-8",
    "org.gradle.vfs.watch": "true",
    "kotlin.incremental": "true",
}
for key, value in managed_properties.items():
    pattern = rf"(?m)^{re.escape(key)}=.*$"
    replacement = f"{key}={value}"
    if re.search(pattern, properties_text):
        properties_text = re.sub(pattern, replacement, properties_text)
    else:
        properties_text += f"\n{replacement}"
gradle_properties.write_text(properties_text.lstrip(), encoding="utf-8")

print(
    f"Android project prepared: API 36, R8/resource shrinking=on, "
    f"signing={'on' if args.signing else 'off'}, professional-icons=on"
)
