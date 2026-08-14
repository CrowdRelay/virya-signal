#!/usr/bin/env python3
from __future__ import annotations

import argparse
import base64
import json
import os
import re
import shutil
import xml.etree.ElementTree as ET
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


def _has_kotlin_named_block(source: str, name: str) -> bool:
    patterns = (
        rf'getByName\(\"{re.escape(name)}\"\)\s*\{{',
        rf'named\(\"{re.escape(name)}\"\)\s*\{{',
        rf'(?m)^[ \t]*{re.escape(name)}[ \t]*\{{',
    )
    return any(re.search(pattern, source) for pattern in patterns)


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


has_debug_build_type = _has_kotlin_named_block(text, "debug")
if has_debug_build_type:
    text = _patch_build_type(text, "debug", minify=False, shrink=None, proguard=False)
text = _patch_build_type(text, "release", minify=True, shrink=True, proguard=True)

# Verify the effective configuration after all mutations.
if has_debug_build_type:
    debug_open, debug_close, _ = _find_kotlin_named_block(text, "debug")
    debug_body = text[debug_open + 1 : debug_close]
    if "isMinifyEnabled = false" not in debug_body:
        raise SystemExit("debug build must remain unminified")
    if "isShrinkResources = true" in debug_body or "proguardFiles(" in debug_body:
        raise SystemExit("release-only shrinker configuration leaked into debug build")
release_open, release_close, _ = _find_kotlin_named_block(text, "release")
release_body = text[release_open + 1 : release_close]
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

    default_source = source_dir / "ic_launcher.xml"
    if not default_source.is_file():
        raise SystemExit(f"missing Tauri-generated adaptive icon: {default_source}")

    for filename in ("ic_launcher.xml", "ic_launcher_round.xml"):
        source = source_dir / filename
        # Tauri may generate only the canonical adaptive icon. Android accepts
        # the same adaptive layers for round launchers, so derive that alias
        # instead of failing the whole APK build.
        if not source.is_file() and filename == "ic_launcher_round.xml":
            source = default_source
        if not source.is_file():
            raise SystemExit(f"missing Tauri-generated adaptive icon: {source}")

        xml = source.read_text(encoding="utf-8")
        if "<adaptive-icon" not in xml or "</adaptive-icon>" not in xml:
            raise SystemExit(f"invalid adaptive icon XML: {source}")

        monochrome = (
            '    <monochrome android:drawable="@mipmap/ic_launcher_foreground" />\n'
        )
        # Keep the operation idempotent in reused local or CI workspaces.
        if "<monochrome" not in xml:
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



# Stage the minimal Android FCM transport into the freshly generated Tauri app.
# Firebase Messaging is the only Firebase runtime dependency: analytics,
# Crashlytics and other SDKs are intentionally absent from Virya Signal.
PUSH_TEMPLATE_DIR = root / "src-tauri" / "android-push"
PUSH_PACKAGE = "music.virya.signal.push"
PUSH_PACKAGE_PATH = Path(*PUSH_PACKAGE.split("."))
FIREBASE_MESSAGING_VERSION = "25.1.1"
GOOGLE_SERVICES_PLUGIN_VERSION = "4.5.0"
ANDROID_NS = "http://schemas.android.com/apk/res/android"
ET.register_namespace("android", ANDROID_NS)


def _find_balanced_block(source: str, marker: str) -> tuple[int, int, str]:
    pattern = r"(?m)^(?P<indent>[ \t]*)" + re.escape(marker) + r"[ \t]*\{"
    match = re.search(pattern, source)
    if match is None:
        raise SystemExit(f"could not locate {marker} block in generated Android Gradle file")
    opening = source.find("{", match.start(), match.end())
    depth = 0
    quote: str | None = None
    escaped = False
    index = opening
    while index < len(source):
        char = source[index]
        if quote is not None:
            if escaped:
                escaped = False
            elif char == "\\":
                escaped = True
            elif char == quote:
                quote = None
            index += 1
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
                return opening, index, match.group("indent")
        index += 1
    raise SystemExit(f"unterminated {marker} block in generated Android Gradle file")


def _insert_in_gradle_block(source: str, marker: str, statement: str) -> str:
    if statement in source:
        return source
    opening, _closing, indent = _find_balanced_block(source, marker)
    return source[: opening + 1] + f"\n{indent}    {statement}" + source[opening + 1 :]


def _ensure_google_plugin_repository() -> None:
    """Ensure Firebase's Gradle plugin can resolve from Google's Maven repo."""
    settings = android / "settings.gradle.kts"
    if not settings.is_file():
        raise SystemExit(f"missing generated Android settings: {settings}")

    settings_text = settings.read_text(encoding="utf-8")
    if not re.search(r"(?m)^[ \t]*pluginManagement[ \t]*\{", settings_text):
        settings_text = (
            "pluginManagement {\n"
            "    repositories {\n"
            "        google()\n"
            "        gradlePluginPortal()\n"
            "        mavenCentral()\n"
            "    }\n"
            "}\n\n"
            + settings_text
        )
    else:
        pm_open, pm_close, pm_indent = _find_balanced_block(settings_text, "pluginManagement")
        pm_body = settings_text[pm_open + 1 : pm_close]
        if not re.search(r"(?m)^[ \t]*google\(\)[ \t]*$", pm_body):
            try:
                repo_open_rel, _repo_close_rel, repo_indent = _find_balanced_block(
                    pm_body, "repositories"
                )
            except SystemExit:
                repository_block = (
                    f"\n{pm_indent}    repositories {{\n"
                    f"{pm_indent}        google()\n"
                    f"{pm_indent}        gradlePluginPortal()\n"
                    f"{pm_indent}        mavenCentral()\n"
                    f"{pm_indent}    }}"
                )
                settings_text = (
                    settings_text[: pm_open + 1]
                    + repository_block
                    + settings_text[pm_open + 1 :]
                )
            else:
                repo_open = pm_open + 1 + repo_open_rel
                settings_text = (
                    settings_text[: repo_open + 1]
                    + f"\n{repo_indent}    google()"
                    + settings_text[repo_open + 1 :]
                )

    settings.write_text(settings_text, encoding="utf-8")


def _stage_android_push() -> bool:
    if not PUSH_TEMPLATE_DIR.is_dir():
        raise SystemExit(f"missing Android push templates: {PUSH_TEMPLATE_DIR}")
    java_dir = android / "app" / "src" / "main" / "java" / PUSH_PACKAGE_PATH
    java_dir.mkdir(parents=True, exist_ok=True)
    for filename in ("SignalPushPlugin.kt", "ViryaFirebaseMessagingService.kt"):
        source = PUSH_TEMPLATE_DIR / filename
        if not source.is_file():
            raise SystemExit(f"missing Android push source: {source}")
        shutil.copy2(source, java_dir / filename)

    manifest = android / "app" / "src" / "main" / "AndroidManifest.xml"
    tree = ET.parse(manifest)
    manifest_root = tree.getroot()
    permission_name = f"{{{ANDROID_NS}}}name"
    if not any(
        node.tag == "uses-permission" and node.attrib.get(permission_name) == "android.permission.POST_NOTIFICATIONS"
        for node in manifest_root
    ):
        ET.SubElement(
            manifest_root,
            "uses-permission",
            {permission_name: "android.permission.POST_NOTIFICATIONS"},
        )
    application = manifest_root.find("application")
    if application is None:
        raise SystemExit("generated Android manifest is missing <application>")
    service_name = f"{PUSH_PACKAGE}.ViryaFirebaseMessagingService"
    existing_service = next(
        (node for node in application.findall("service") if node.attrib.get(permission_name) == service_name),
        None,
    )
    if existing_service is None:
        service = ET.SubElement(
            application,
            "service",
            {
                permission_name: service_name,
                f"{{{ANDROID_NS}}}exported": "false",
            },
        )
        intent_filter = ET.SubElement(service, "intent-filter")
        ET.SubElement(
            intent_filter,
            "action",
            {permission_name: "com.google.firebase.MESSAGING_EVENT"},
        )
    tree.write(manifest, encoding="utf-8", xml_declaration=True)

    gradle_text = gradle.read_text(encoding="utf-8")
    gradle_text = _insert_in_gradle_block(
        gradle_text,
        "dependencies",
        f'implementation("com.google.firebase:firebase-messaging:{FIREBASE_MESSAGING_VERSION}")',
    )

    firebase_b64 = os.environ.get("VIRYA_SIGNAL_GOOGLE_SERVICES_JSON_B64", "").strip()
    google_services = android / "app" / "google-services.json"
    configured = False
    config_sha = None
    if firebase_b64:
        try:
            raw = base64.b64decode(firebase_b64, validate=True)
        except Exception as error:
            raise SystemExit(f"invalid VIRYA_SIGNAL_GOOGLE_SERVICES_JSON_B64: {error}") from error
        if not (64 <= len(raw) <= 128 * 1024):
            raise SystemExit("google-services.json has an invalid size")
        try:
            document = json.loads(raw)
        except json.JSONDecodeError as error:
            raise SystemExit(f"invalid google-services.json: {error}") from error
        project_id = str(document.get("project_info", {}).get("project_id", "")).strip()
        packages = {
            str(client.get("client_info", {}).get("android_client_info", {}).get("package_name", "")).strip()
            for client in document.get("client", [])
            if isinstance(client, dict)
        }
        if not project_id or "music.virya.control" not in packages:
            raise SystemExit("google-services.json does not target music.virya.control")
        google_services.write_bytes(raw)
        _ensure_google_plugin_repository()
        gradle_text = _insert_in_gradle_block(
            gradle_text,
            "plugins",
            f'id("com.google.gms.google-services") version "{GOOGLE_SERVICES_PLUGIN_VERSION}"',
        )
        import hashlib
        config_sha = hashlib.sha256(raw).hexdigest()
        configured = True
    else:
        google_services.unlink(missing_ok=True)

    gradle.write_text(gradle_text, encoding="utf-8")
    receipt = {
        "schemaVersion": 1,
        "firebaseConfigured": configured,
        "firebaseConfigSha256": config_sha,
        "firebaseMessagingVersion": FIREBASE_MESSAGING_VERSION,
        "googleServicesPluginVersion": GOOGLE_SERVICES_PLUGIN_VERSION if configured else None,
        "analyticsIncluded": False,
        "crashlyticsIncluded": False,
    }
    (android / "push-build-config.json").write_text(
        json.dumps(receipt, sort_keys=True, indent=2) + "\n",
        encoding="utf-8",
    )
    return configured


push_configured = _stage_android_push()

print(
    f"Android project prepared: API 36, R8/resource shrinking=on, "
    f"signing={'on' if args.signing else 'off'}, tauri-icons=on, debug-r8=off, "
    f"push-firebase={'on' if push_configured else 'degraded-no-config'}"
)
