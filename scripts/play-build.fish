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

# Przywróć środowisko Androida również w świeżym terminalu.
set -gx ANDROID_HOME "$HOME/Library/Android/sdk"

if not set -q JAVA_HOME
    if brew list --formula openjdk@21 >/dev/null 2>&1
        set -gx JAVA_HOME (brew --prefix openjdk@21)/libexec/openjdk.jdk/Contents/Home
    end
end

if not set -q NDK_HOME
    set -gx NDK_HOME "$ANDROID_HOME/ndk/29.0.14206865"
end

test -x "$JAVA_HOME/bin/java"; or begin
    echo "ERROR: invalid JAVA_HOME=$JAVA_HOME" >&2
    exit 1
end

test -d "$ANDROID_HOME/platforms/android-36"; or begin
    echo "ERROR: Android API 36 missing" >&2
    exit 1
end

test -d "$NDK_HOME"; or begin
    echo "ERROR: invalid NDK_HOME=$NDK_HOME" >&2
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

cargo tauri android build --aab
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