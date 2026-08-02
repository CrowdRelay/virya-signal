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

text = gradle.read_text(encoding="utf-8")
text, compile_count = re.subn(r"\bcompileSdk\s*=\s*\d+", "compileSdk = 36", text)
text, target_count = re.subn(r"\btargetSdk\s*=\s*\d+", "targetSdk = 36", text)
if compile_count != 1 or target_count != 1:
    raise SystemExit(
        "expected exactly one compileSdk and targetSdk in generated Gradle file "
        f"(found {compile_count}/{target_count})"
    )

# Release APK/AAB builds should ship neither unused CameraX/ML Kit bytecode nor
# dead Android resources. Debug APKs must remain unminified: Android/Tauri
# plugins use generated entry points that are unsafe to shrink in smoke builds.
def _find_kotlin_named_block(source: str, name: str) -> tuple[int, int, str]:
    patterns = (
        rf'getByName\(\"{re.escape(name)}\"\)\s*\{{',
        rf'named\(\"{re.escape(name)}\"\)\s*\{{',
        rf'(?m)^(?P<indent>\s*){re.escape(name)}\s*\{{',
    )
    match = None
    for pattern in patterns:
        match = re.search(pattern, source)
        if match:
            break
    if match is None:
        raise SystemExit(f"could not locate {name} build type in generated Gradle file")

    opening = source.find("{", match.start(), match.end())
    if opening < 0:
        raise SystemExit(f"could not locate opening brace for {name} build type")

    depth = 0
    quote: str | None = None
    escaped = False
    line_comment = False
    block_comment = False
    index = opening
    while index < len(source):
        char = source[index]
        next_char = source[index + 1] if index + 1 < len(source) else ""
        if line_comment:
            if char == "\n":
                line_comment = False
            index += 1
            continue
        if block_comment:
            if char == "*" and next_char == "/":
                block_comment = False
                index += 2
            else:
                index += 1
            continue
        if quote is not None:
            if escaped:
                escaped = False
            elif char == "\\":
                escaped = True
            elif char == quote:
                quote = None
            index += 1
            continue
        if char == "/" and next_char == "/":
            line_comment = True
            index += 2
            continue
        if char == "/" and next_char == "*":
            block_comment = True
            index += 2
            continue
        if char in ('"', "'"):
            quote = char
            index += 1
            continue
        if char == "{":
            depth += 1
        elif char == "}":
            depth -= 1
            if depth == 0:
                line_start = source.rfind("\n", 0, match.start()) + 1
                indent = source[line_start:match.start()]
                return opening, index, indent
        index += 1
    raise SystemExit(f"unterminated {name} build type in generated Gradle file")


def _set_kotlin_property(body: str, key: str, value: str, indent: str) -> str:
    pattern = rf'(?m)^[ \t]*{re.escape(key)}[ \t]*=[ \t]*(?:true|false)[ \t]*$'
    replacement = f"{indent}{key} = {value}"
    if re.search(pattern, body):
        return re.sub(pattern, replacement, body, count=1)
    return f"\n{replacement}" + body


def _patch_build_type(
    source: str,
    name: str,
    *,
    minify: bool,
    shrink: bool | None,
    proguard: bool,
) -> str:
    opening, closing, base_indent = _find_kotlin_named_block(source, name)
    indent = base_indent + "    "
    body = source[opening + 1 : closing]
    body = _set_kotlin_property(body, "isMinifyEnabled", str(minify).lower(), indent)

    shrink_pattern = r'(?m)^[ \t]*isShrinkResources[ \t]*=[ \t]*(?:true|false)[ \t]*\n?'
    if shrink is None:
        body = re.sub(shrink_pattern, "", body)
    else:
        body = _set_kotlin_property(body, "isShrinkResources", str(shrink).lower(), indent)

    proguard_pattern = (
        r'(?m)^\s*proguardFiles\(getDefaultProguardFile\('
        r'\"proguard-android-optimize\.txt\"\),\s*\"proguard-rules\.pro\"\)\s*\n?'
    )
    body = re.sub(proguard_pattern, "", body)
    if proguard:
        body = (
            f'\n{indent}proguardFiles(getDefaultProguardFile("proguard-android-optimize.txt"), '
            f'"proguard-rules.pro")' + body
        )

    return source[: opening + 1] + body + source[closing:]


text = _patch_build_type(text, "debug", minify=False, shrink=None, proguard=False)
text = _patch_build_type(text, "release", minify=True, shrink=True, proguard=True)

# Verify the effective configuration after all mutations.
debug_open, debug_close, _ = _find_kotlin_named_block(text, "debug")
debug_body = text[debug_open + 1 : debug_close]
release_open, release_close, _ = _find_kotlin_named_block(text, "release")
release_body = text[release_open + 1 : release_close]
if "isMinifyEnabled = false" not in debug_body:
    raise SystemExit("debug build must remain unminified")
if "isShrinkResources = true" in debug_body or "proguardFiles(" in debug_body:
    raise SystemExit("release-only shrinker configuration leaked into debug build")
for fragment in (
    "isMinifyEnabled = true",
    "isShrinkResources = true",
    'proguardFiles(getDefaultProguardFile("proguard-android-optimize.txt"), "proguard-rules.pro")',
):
    if fragment not in release_body:
        raise SystemExit(f"release build invariant missing: {fragment}")

# Each Kotlin shrinker property must remain on its own physical line. The
# property matcher above intentionally uses horizontal whitespace only because
# Python's ``\s`` also matches newlines.
for line_number, line in enumerate(text.splitlines(), start=1):
    property_count = line.count("isMinifyEnabled") + line.count("isShrinkResources")
    if property_count > 1:
        raise SystemExit(
            f"malformed Gradle build type at line {line_number}: "
            "minify/shrink assignments were joined"
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

# Android 13+ themed icons require a v33 adaptive-icon resource with an
# explicit monochrome layer. Reuse the foreground generated immediately before
# this script by `cargo tauri icon`; never replace it with stale launcher assets.
MONOCHROME_ADAPTIVE_ICON_DIR = "mipmap-anydpi-v33"


def _install_monochrome_adaptive_icons() -> None:
    resources = android / "app" / "src" / "main" / "res"
    source_dir = resources / "mipmap-anydpi-v26"
    target_dir = resources / MONOCHROME_ADAPTIVE_ICON_DIR
    target_dir.mkdir(parents=True, exist_ok=True)

    for filename in ("ic_launcher.xml", "ic_launcher_round.xml"):
        source = source_dir / filename
        if not source.is_file():
            raise SystemExit(f"missing Tauri-generated adaptive icon: {source}")
        xml = source.read_text(encoding="utf-8")
        if "<adaptive-icon" not in xml or "</adaptive-icon>" not in xml:
            raise SystemExit(f"invalid adaptive icon XML: {source}")

        monochrome = (
            '    <monochrome android:drawable="@mipmap/ic_launcher_foreground" />\n'
        )
        xml = xml.replace("</adaptive-icon>", monochrome + "</adaptive-icon>", 1)
        if xml.count("<monochrome") != 1:
            raise SystemExit(f"invalid monochrome layer count for {filename}")

        target = target_dir / filename
        target.write_text(xml, encoding="utf-8")


_install_monochrome_adaptive_icons()

# Launcher resources are owned by the preceding `cargo tauri icon` step.
# Never overwrite them here with stale hand-copied Android assets.
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
    f"signing={'on' if args.signing else 'off'}, tauri-icons=on, debug-r8=off"
)
