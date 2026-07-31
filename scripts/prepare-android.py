#!/usr/bin/env python3
from __future__ import annotations

import argparse
import re
from pathlib import Path

parser = argparse.ArgumentParser()
parser.add_argument("--signing", action="store_true")
args = parser.parse_args()

root = Path(__file__).resolve().parents[1]
android = root / "src-tauri" / "gen" / "android"
gradle = android / "app" / "build.gradle.kts"
if not gradle.is_file():
    raise SystemExit(f"missing generated Android project: {gradle}")

text = gradle.read_text()
text, compile_count = re.subn(r"\bcompileSdk\s*=\s*\d+", "compileSdk = 36", text)
text, target_count = re.subn(r"\btargetSdk\s*=\s*\d+", "targetSdk = 36", text)
if compile_count == 0 or target_count == 0:
    raise SystemExit("could not locate compileSdk/targetSdk in generated Gradle file")

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

gradle.write_text(text)
print(f"Android project prepared: API 36, signing={'on' if args.signing else 'off'}")
