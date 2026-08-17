#!/usr/bin/env fish
set -l package music.virya.signal
set -l stamp (date +%Y%m%d-%H%M%S)
set -l out "$HOME/Downloads/virya-signal-startup-$stamp.log"
set -l raw "$HOME/Downloads/virya-signal-startup-$stamp.raw.log"

if not type -q adb
    echo "Brak adb w PATH." >&2
    exit 1
end

adb wait-for-device; or exit 1

begin
    echo "Virya Signal Android startup diagnostics"
    echo "timestamp="(date "+%Y-%m-%dT%H:%M:%S%z")
    echo "package=$package"
    echo
    echo "== DEVICE =="
    adb shell getprop ro.product.manufacturer
    adb shell getprop ro.product.model
    echo -n "android="; adb shell getprop ro.build.version.release
    echo -n "sdk="; adb shell getprop ro.build.version.sdk
    echo -n "abi="; adb shell getprop ro.product.cpu.abi
    echo -n "page_size="; adb shell getconf PAGE_SIZE 2>/dev/null; or true
    echo
    echo "== PACKAGE =="
    adb shell dumpsys package $package 2>/dev/null | grep -E 'versionName=|versionCode=|primaryCpuAbi=|debuggable|targetSdk='; or true
    echo -n "activity="
    adb shell cmd package resolve-activity --brief $package 2>/dev/null; or true
end > $out

adb logcat -c >/dev/null 2>&1; or true
adb shell am force-stop $package >/dev/null 2>&1; or true

adb logcat -b all -v threadtime > $raw &
set -l logger_pid $last_pid
sleep 1

# Launch via the package launcher intent so the diagnostic does not depend on
# the generated Activity class name.
adb shell monkey -p $package -c android.intent.category.LAUNCHER 1 >/dev/null 2>&1; or true
sleep 8

set -l app_pid (adb shell pidof $package 2>/dev/null | string trim)

kill $logger_pid 2>/dev/null; or true
wait $logger_pid 2>/dev/null; or true

begin
    echo
    echo "== PROCESS AFTER 8s =="
    if test -n "$app_pid"
        echo "alive pid=$app_pid"
    else
        echo "DEAD"
    end

    echo
    echo "== ANDROID EXIT INFO =="
    adb shell dumpsys activity exit-info $package 2>/dev/null; or true

    echo
    echo "== CRASH BUFFER =="
    adb logcat -b crash -d -v threadtime 2>/dev/null; or true

    echo
    echo "== MATCHED RAW LOG =="
    grep -Ei \
        'FATAL EXCEPTION|Fatal signal|AndroidRuntime|UnsatisfiedLinkError|dlopen|linker|JNI|SIGABRT|SIGSEGV|SIGBUS|Abort message|am_crash|am_kill|has died|Process .* died|chromium|WebView|crash|panic|wasm|tauri|virya|crowdrelay' \
        $raw; or true

    echo
    echo "== APP-PERSISTED NATIVE REPORT =="
    adb shell run-as $package sh -c \
        'for f in $(find . -type f -name last-native-crash-v2.txt 2>/dev/null); do echo "--- $f"; cat "$f"; done' \
        2>/dev/null; or echo "not available (no report or package is not debuggable)"

    echo
    echo "== RECENT DROPBOX CRASHES =="
    adb shell dumpsys dropbox --print data_app_crash 2>/dev/null | tail -n 250; or true
end >> $out

echo "Gotowe: $out"
echo "Surowy logcat: $raw"
if test -z "$app_pid"
    echo "Proces aplikacji nie przeżył 8 sekund — sekcja ANDROID EXIT INFO jest najważniejsza."
end
