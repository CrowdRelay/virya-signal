#!/usr/bin/env python3
from __future__ import annotations

import argparse
import base64
import json
import sys
import os
import re
import shutil
import xml.etree.ElementTree as ET
from pathlib import Path

parser = argparse.ArgumentParser()
parser.add_argument("--signing", action="store_true")
args = parser.parse_args()

root = Path(__file__).resolve().parents[1]
# Tauri's Android project root is src-tauri/gen/android. The app snake-case
# directory mentioned by `tauri android init` is used for generated/native
# paths, but it is not the Gradle project root.
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

# Keep Android release startup conservative until the release artifact itself
# is runtime-smoked. Tauri/plugin generated entry points have previously been
# exercised only by unminified debug E2E; enabling R8 without a release-runtime
# gate can therefore turn a structurally valid Play AAB into an instant crash.
# Size optimisation can be re-enabled later behind that gate.
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



def _remove_kotlin_call_statements(source: str, name: str) -> str:
    # Remove complete Kotlin call statements such as proguardFiles(...).
    # Handles nested parentheses and multiline generated Gradle formatting.
    pattern = re.compile(rf'(?m)^[ \t]*{re.escape(name)}[ \t]*\(')

    while True:
        match = pattern.search(source)
        if match is None:
            return source

        opening = source.find("(", match.start(), match.end())
        if opening < 0:
            raise SystemExit(f"could not locate opening parenthesis for {name}")

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

            if char == "(":
                depth += 1
            elif char == ")":
                depth -= 1
                if depth == 0:
                    end = index + 1
                    while end < len(source) and source[end] in " \t":
                        end += 1
                    if end < len(source) and source[end] == "\n":
                        end += 1
                    source = source[:match.start()] + source[end:]
                    break

            index += 1
        else:
            raise SystemExit(f"unterminated Kotlin call: {name}(...)")


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

    body = _remove_kotlin_call_statements(body, "proguardFiles")
    if proguard:
        body = (
            f'\n{indent}proguardFiles(getDefaultProguardFile("proguard-android-optimize.txt"), '
            f'"proguard-rules.pro")' + body
        )

    return source[: opening + 1] + body + source[closing:]


has_debug_build_type = _has_kotlin_named_block(text, "debug")
if has_debug_build_type:
    text = _patch_build_type(text, "debug", minify=False, shrink=None, proguard=False)
text = _patch_build_type(text, "release", minify=False, shrink=False, proguard=False)

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
    "isMinifyEnabled = false",
    "isShrinkResources = false",
):
    if fragment not in release_body:
        raise SystemExit(f"release build invariant missing: {fragment}")
if "proguardFiles(" in release_body:
    raise SystemExit("release build must not enable ProGuard while the safe release mode is active")

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

# Tauri's generic icon generator uses the full app tile as the adaptive
# foreground, which makes the Android foreground fully opaque. Keep one audited
# platform-specific source of truth instead: legacy launchers use the full tile,
# while adaptive foregrounds use the transparent Signal Core artwork.
ANDROID_ICON_SOURCE_DIR = root / "src-tauri" / "icons" / "android"
MONOCHROME_ADAPTIVE_ICON_DIR = "mipmap-anydpi-v33"


def _install_android_launcher_assets() -> None:
    resources = android / "app" / "src" / "main" / "res"
    required = (
        ANDROID_ICON_SOURCE_DIR / "mipmap-anydpi-v26" / "ic_launcher.xml",
        ANDROID_ICON_SOURCE_DIR / "values" / "ic_launcher_background.xml",
        ANDROID_ICON_SOURCE_DIR / "mipmap-xxxhdpi" / "ic_launcher.png",
        ANDROID_ICON_SOURCE_DIR / "mipmap-xxxhdpi" / "ic_launcher_round.png",
        ANDROID_ICON_SOURCE_DIR / "mipmap-xxxhdpi" / "ic_launcher_foreground.png",
    )
    missing = [str(path) for path in required if not path.is_file()]
    if missing:
        raise SystemExit(f"missing canonical Android launcher assets: {', '.join(missing)}")

    for source_dir in sorted(path for path in ANDROID_ICON_SOURCE_DIR.iterdir() if path.is_dir()):
        target_dir = resources / source_dir.name
        target_dir.mkdir(parents=True, exist_ok=True)
        for source in sorted(source_dir.iterdir()):
            if source.is_file():
                shutil.copy2(source, target_dir / source.name)


def _install_monochrome_adaptive_icons() -> None:
    resources = android / "app" / "src" / "main" / "res"
    source_dir = resources / "mipmap-anydpi-v26"
    target_dir = resources / MONOCHROME_ADAPTIVE_ICON_DIR
    target_dir.mkdir(parents=True, exist_ok=True)

    default_source = source_dir / "ic_launcher.xml"
    if not default_source.is_file():
        raise SystemExit(f"missing canonical adaptive icon: {default_source}")

    for filename in ("ic_launcher.xml", "ic_launcher_round.xml"):
        source = source_dir / filename
        if not source.is_file() and filename == "ic_launcher_round.xml":
            source = default_source
        if not source.is_file():
            raise SystemExit(f"missing canonical adaptive icon: {source}")

        xml = source.read_text(encoding="utf-8")
        if "<adaptive-icon" not in xml or "</adaptive-icon>" not in xml:
            raise SystemExit(f"invalid adaptive icon XML: {source}")

        monochrome = (
            '    <monochrome android:drawable="@mipmap/ic_launcher_foreground" />\n'
        )
        if "<monochrome" not in xml:
            xml = xml.replace("</adaptive-icon>", monochrome + "</adaptive-icon>", 1)
        if xml.count("<monochrome") != 1:
            raise SystemExit(f"invalid monochrome layer count for {filename}")

        target = target_dir / filename
        target.write_text(xml, encoding="utf-8")


_install_android_launcher_assets()
_install_monochrome_adaptive_icons()

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


def _application_id() -> str:
    """Read the Android application ID from its single source of truth.

    A Play application ID is permanent identity after the first upload, so it
    must never be re-declared per script or per workflow where the copies can
    drift apart. tauri.conf.json is what actually generates the Android project.
    """
    conf = json.loads((root / "src-tauri" / "tauri.conf.json").read_text(encoding="utf-8"))
    identifier = str(conf.get("identifier", "")).strip()
    if not re.fullmatch(r"[a-z][a-z0-9_]*(\.[a-z][a-z0-9_]*)+", identifier):
        raise SystemExit(f"tauri.conf.json identifier is not a valid Android application ID: {identifier!r}")
    return identifier


APPLICATION_ID = _application_id()
PUSH_PACKAGE = f"{APPLICATION_ID}.push"
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
    # Tauri 2.x generates Groovy settings.gradle today. Keep Kotlin DSL support
    # as a compatibility fallback, but never assume one extension.
    settings_candidates = (
        android / "settings.gradle",
        android / "settings.gradle.kts",
    )
    settings = next((candidate for candidate in settings_candidates if candidate.is_file()), None)
    if settings is None:
        expected = ", ".join(str(candidate) for candidate in settings_candidates)
        raise SystemExit(f"missing generated Android settings; checked: {expected}")

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

    notification_icon = PUSH_TEMPLATE_DIR / "virya_signal_notification.xml"
    if not notification_icon.is_file():
        raise SystemExit(f"missing Android notification icon: {notification_icon}")
    drawable_dir = android / "app" / "src" / "main" / "res" / "drawable"
    drawable_dir.mkdir(parents=True, exist_ok=True)
    shutil.copy2(notification_icon, drawable_dir / notification_icon.name)

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

    # Verified HTTPS App Links are the canonical Latarnik invitation transport.
    # The token remains a one-time CrowdRelay capability; Android only routes
    # the trusted virya.music URL into the native shell.
    # The filter has to land on the activity Android actually starts for a VIEW
    # intent, so prefer the declared launcher rather than manifest order.
    activities = [node for node in application.findall("activity") if node.attrib.get(permission_name)]
    activity = next(
        (
            node
            for node in activities
            if any(
                category.attrib.get(permission_name) == "android.intent.category.LAUNCHER"
                for intent_filter in node.findall("intent-filter")
                for category in intent_filter.findall("category")
            )
        ),
        activities[0] if len(activities) == 1 else None,
    )
    if activity is None:
        raise SystemExit("generated Android manifest is missing an unambiguous launcher activity")
    app_link_filter = next(
        (node for node in activity.findall("intent-filter")
         if node.attrib.get(f"{{{ANDROID_NS}}}autoVerify") == "true"
         and any(data.attrib.get(f"{{{ANDROID_NS}}}host") == "virya.music" for data in node.findall("data"))),
        None,
    )
    if app_link_filter is None:
        app_link_filter = ET.SubElement(
            activity, "intent-filter", {f"{{{ANDROID_NS}}}autoVerify": "true"}
        )

    def ensure_intent_child(tag: str, name: str) -> None:
        if not any(
            node.attrib.get(permission_name) == name
            for node in app_link_filter.findall(tag)
        ):
            ET.SubElement(app_link_filter, tag, {permission_name: name})

    ensure_intent_child("action", "android.intent.action.VIEW")
    ensure_intent_child("category", "android.intent.category.DEFAULT")
    ensure_intent_child("category", "android.intent.category.BROWSABLE")

    # Verify only the canonical apex host. Android requires every declared host
    # to serve its own assetlinks.json without redirects. Reconcile each path on
    # every run because generated Android trees can survive across source
    # upgrades and may already contain an older Latarnik-only verified filter.
    for path_prefix in ("/latarnik", "/pl/latarnik", "/my-signal", "/pl/my-signal"):
        expected = {
            f"{{{ANDROID_NS}}}scheme": "https",
            f"{{{ANDROID_NS}}}host": "virya.music",
            f"{{{ANDROID_NS}}}pathPrefix": path_prefix,
        }
        if not any(
            all(data.attrib.get(key) == value for key, value in expected.items())
            for data in app_link_filter.findall("data")
        ):
            ET.SubElement(app_link_filter, "data", expected)
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
        if not project_id or APPLICATION_ID not in packages:
            raise SystemExit(
                f"google-services.json does not target {APPLICATION_ID}; "
                "register the Android app for this application ID in Firebase and "
                "download a fresh google-services.json"
            )
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
    elif args.signing:
        # A --signing run produces a shippable artifact. Dropping the Firebase
        # wiring here yields an APK/AAB whose only symptom is the native plugin
        # rejecting with "firebase_not_configured" the first time an operator
        # touches push, long after the build looked successful. Refuse instead.
        raise SystemExit(
            "VIRYA_SIGNAL_GOOGLE_SERVICES_JSON_B64 is required for a signed build: "
            "without it the artifact ships with push permanently broken. Export it, "
            "or drop --signing for a local build without push."
        )
    else:
        google_services.unlink(missing_ok=True)
        print(
            "WARNING: no VIRYA_SIGNAL_GOOGLE_SERVICES_JSON_B64; this build has no "
            "Firebase wiring and push will reject with firebase_not_configured",
            file=sys.stderr,
        )

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
    f"Android project prepared: API 36, R8/resource shrinking=off-safe-release, "
    f"signing={'on' if args.signing else 'off'}, canonical-icons=on, debug-r8=off, "
    f"push-firebase={'on' if push_configured else 'degraded-no-config'}"
)
