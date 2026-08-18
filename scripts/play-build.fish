#!/usr/bin/env fish

set -l ROOT (git rev-parse --show-toplevel 2>/dev/null)
or begin
    echo "ERROR: not inside a git repository" >&2
    exit 1
end

cd "$ROOT"; or exit 1

set -l CONFIG "src-tauri/tauri.conf.json"
set -l PROPS "src-tauri/gen/android/keystore.properties"
set -l AAB "src-tauri/gen/android/app/build/outputs/bundle/universalRelease/app-universal-release.aab"

test -f "$CONFIG"; or begin
    echo "ERROR: missing $CONFIG" >&2
    exit 1
end

test -f "$PROPS"; or begin
    echo "ERROR: missing signing config: $PROPS" >&2
    exit 1
end

# Resolve and verify the JDK before anything else. Gradle fails deep in the
# build with a poor message when JAVA_HOME is wrong or absent, and inheriting
# whatever the shell happens to carry is how the wrong JDK gets used silently.
set -l JAVA21 ""

if brew list --formula openjdk@21 >/dev/null 2>&1
    set JAVA21 (brew --prefix openjdk@21)/libexec/openjdk.jdk/Contents/Home
else
    set JAVA21 (/usr/libexec/java_home -v 21 2>/dev/null)
end

if test -z "$JAVA21"; or not test -x "$JAVA21/bin/java"
    echo "ERROR: JDK 21 not found" >&2
    echo "Install with: brew install openjdk@21" >&2
    exit 1
end

set -gx JAVA_HOME "$JAVA21"

set -l JAVA_MAJOR ("$JAVA_HOME/bin/java" -version 2>&1 | string match -r 'version "[0-9]+' | string replace 'version "' '')

if test "$JAVA_MAJOR" != "21"
    echo "ERROR: Signal requires JDK 21, found $JAVA_MAJOR" >&2
    "$JAVA_HOME/bin/java" -version >&2
    exit 1
end

echo "JAVA_HOME=$JAVA_HOME"

# Przywróć środowisko Androida również w świeżym terminalu.
set -gx ANDROID_HOME "$HOME/Library/Android/sdk"
set -gx ANDROID_SDK_ROOT "$ANDROID_HOME"

set -gx NDK_HOME "$ANDROID_HOME/ndk/29.0.14206865"
set -gx ANDROID_NDK_HOME "$NDK_HOME"

test -d "$ANDROID_HOME/platforms/android-36"; or begin
    echo "ERROR: Android API 36 missing: $ANDROID_HOME/platforms/android-36" >&2
    exit 1
end

test -d "$NDK_HOME"; or begin
    echo "ERROR: Signal NDK missing: $NDK_HOME" >&2
    echo "Available NDKs:" >&2
    ls "$ANDROID_HOME/ndk" 2>/dev/null
    exit 1
end

# Local Play builds must have the same Firebase guarantee as signed CI builds.
# Keep the config outside the generated Tauri tree, because `src-tauri/gen` is
# disposable and can be recreated by `tauri android init`.
set -l GOOGLE_SERVICES "src-tauri/gen/android/app/google-services.json"
set -l LOCAL_FIREBASE "$HOME/.config/virya-signal/google-services.json"

if not set -q VIRYA_SIGNAL_GOOGLE_SERVICES_JSON_B64
    if test -s "$GOOGLE_SERVICES"
        set -gx VIRYA_SIGNAL_GOOGLE_SERVICES_JSON_B64 (base64 < "$GOOGLE_SERVICES" | tr -d '\n')
    else if test -s "$LOCAL_FIREBASE"
        set -gx VIRYA_SIGNAL_GOOGLE_SERVICES_JSON_B64 (base64 < "$LOCAL_FIREBASE" | tr -d '\n')
    end
end

if not set -q VIRYA_SIGNAL_GOOGLE_SERVICES_JSON_B64; or test -z "$VIRYA_SIGNAL_GOOGLE_SERVICES_JSON_B64"
    echo "ERROR: Firebase config missing for signed Play build" >&2
    echo "Expected: $LOCAL_FIREBASE" >&2
    echo "or export VIRYA_SIGNAL_GOOGLE_SERVICES_JSON_B64 before building." >&2
    exit 1
end

python3 scripts/prepare-android.py --signing
or begin
    echo "ERROR: canonical Android preparation failed" >&2
    exit 1
end

python3 -c 'import json; from pathlib import Path; p=Path("src-tauri/gen/android/push-build-config.json"); d=json.loads(p.read_text()); assert d.get("firebaseConfigured") is True, "signed Play build is missing Firebase configuration"; print("SIGNAL_LOCAL_PLAY_PUSH_BUILD_GATE=PASS firebase=true")'
or begin
    echo "ERROR: signed Play build failed Firebase configuration gate" >&2
    exit 1
end

# Odczytaj obecny versionCode.
set -l OLD_CODE (python3 -c 'import json; d=json.load(open("src-tauri/tauri.conf.json")); print(d["bundle"]["android"]["versionCode"])')
or exit 1

set -l NEW_CODE (math "$OLD_CODE + 1")

# Podbij versionCode.
python3 -c 'import json,sys; p="src-tauri/tauri.conf.json"; d=json.load(open(p)); d["bundle"]["android"]["versionCode"]=int(sys.argv[1]); open(p,"w").write(json.dumps(d,indent=2,ensure_ascii=False)+"\n")' "$NEW_CODE"
or exit 1

set -l VERSION_NAME (python3 -c 'import json; d=json.load(open("src-tauri/tauri.conf.json")); print(d["version"])')

echo
echo "PLAY_VERSION=$VERSION_NAME"
echo "VERSION_CODE=$OLD_CODE -> $NEW_CODE"
echo "BUILD=START"
echo

cargo tauri android build --aab --target aarch64
set -l BUILD_STATUS $status

# Nie marnujemy versionCode, jeśli sam build nie powstał.
if test $BUILD_STATUS -ne 0
    python3 -c 'import json,sys; p="src-tauri/tauri.conf.json"; d=json.load(open(p)); d["bundle"]["android"]["versionCode"]=int(sys.argv[1]); open(p,"w").write(json.dumps(d,indent=2,ensure_ascii=False)+"\n")' "$OLD_CODE"

    echo
    echo "BUILD=FAIL"
    echo "VERSION_CODE=RESTORED_TO_$OLD_CODE"
    exit $BUILD_STATUS
end

test -s "$AAB"; or begin
    echo "ERROR: AAB missing after successful build" >&2
    exit 1
end

python3 scripts/analyze-android-package.py "$AAB" --require-abi arm64-v8a --require-page-size 16384
or begin
    python3 -c 'import json,sys; p="src-tauri/tauri.conf.json"; d=json.load(open(p)); d["bundle"]["android"]["versionCode"]=int(sys.argv[1]); open(p,"w").write(json.dumps(d,indent=2,ensure_ascii=False)+"\n")' "$OLD_CODE"
    echo "ERROR: release AAB structural/page-size gate failed" >&2
    echo "VERSION_CODE=RESTORED_TO_$OLD_CODE"
    exit 1
end

python3 scripts/check-android-firebase-artifact.py "$AAB"
or begin
    python3 -c 'import json,sys; p="src-tauri/tauri.conf.json"; d=json.load(open(p)); d["bundle"]["android"]["versionCode"]=int(sys.argv[1]); open(p,"w").write(json.dumps(d,indent=2,ensure_ascii=False)+"\n")' "$OLD_CODE"
    echo "ERROR: release AAB Firebase runtime gate failed" >&2
    echo "VERSION_CODE=RESTORED_TO_$OLD_CODE"
    exit 1
end

python3 scripts/check-android-app-links-artifact.py "$AAB"
or begin
    python3 -c 'import json,sys; p="src-tauri/tauri.conf.json"; d=json.load(open(p)); d["bundle"]["android"]["versionCode"]=int(sys.argv[1]); open(p,"w").write(json.dumps(d,indent=2,ensure_ascii=False)+"\n")' "$OLD_CODE"
    echo "ERROR: release AAB App Links gate failed" >&2
    echo "VERSION_CODE=RESTORED_TO_$OLD_CODE"
    exit 1
end

echo
echo "=== PLAY BUILD READY ==="
echo "versionName=$VERSION_NAME"
echo "versionCode=$NEW_CODE"
ls -lh "$AAB"

echo
echo "=== SHA256 ==="
shasum -a 256 "$AAB"

echo
echo "BUILD=PASS"
echo "AAB=$AAB"