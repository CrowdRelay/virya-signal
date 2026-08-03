#!/usr/bin/env fish
set -l out "$HOME/Downloads/virya-signal-crash-"(date +%Y%m%d-%H%M%S)".log"

if not type -q adb
    echo "Brak adb w PATH." >&2
    exit 1
end

adb wait-for-device
adb logcat -c
echo "Loguję przez 35 sekund do: $out"
echo "Teraz otwórz Virya Signal i odtwórz crash."
adb logcat -v threadtime > "$out" &
set -l pid $last_pid
sleep 35
kill $pid 2>/dev/null
wait $pid 2>/dev/null

grep -Ei "FATAL EXCEPTION|AndroidRuntime|chromium|crash|panic|wasm|virya|crowdrelay" "$out" > "$out.filtered"; or true
echo "Gotowe: $out.filtered"
