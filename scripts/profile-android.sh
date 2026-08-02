#!/usr/bin/env bash
set -euo pipefail

package_name="${1:-music.virya.control}"
profile_dir="${2:-artifacts/device-profile-$(date +%Y%m%d-%H%M%S)}"
run_count="${3:-5}"
activity_name="${package_name}/.MainActivity"

command -v adb >/dev/null 2>&1 || { echo "adb is required" >&2; exit 1; }
connected_devices="$(adb devices | awk 'NR > 1 && $2 == "device" { count++ } END { print count + 0 }')"
if [[ "$connected_devices" != "1" ]]; then
  echo "connect exactly one unlocked Android device (found $connected_devices)" >&2
  exit 1
fi
if ! [[ "$run_count" =~ ^[1-9][0-9]*$ ]] || ((run_count > 20)); then
  echo "run count must be between 1 and 20" >&2
  exit 1
fi

mkdir -p "$profile_dir"
adb shell am force-stop "$package_name"
adb shell dumpsys gfxinfo "$package_name" reset >/dev/null 2>&1 || true
adb logcat -c

# One warm-up avoids charging dex extraction/first install work to every sample.
adb shell am start -W -n "$activity_name" >/dev/null
adb shell am force-stop "$package_name"
for run in $(seq 1 "$run_count"); do
  adb shell am start -W -n "$activity_name" | tee "$profile_dir/cold-start-$run.txt"
  adb shell am force-stop "$package_name"
done
adb shell am start -W -n "$activity_name" >/dev/null
sleep 3
adb shell dumpsys meminfo "$package_name" > "$profile_dir/memory.txt"
adb shell dumpsys gfxinfo "$package_name" framestats > "$profile_dir/frames.txt"
adb shell dumpsys package "$package_name" > "$profile_dir/package.txt"
pid="$(adb shell pidof "$package_name" | tr -d '\r')"
if [[ -n "$pid" ]]; then
  adb shell top -b -n 1 -p "$pid" > "$profile_dir/cpu.txt" || true
fi
adb logcat -d -v threadtime | grep -E '\[virya:(boot|ipc)\]|chromium' > "$profile_dir/startup-log.txt" || true

awk '/TotalTime:/ { print $2 }' "$profile_dir"/cold-start-*.txt | sort -n > "$profile_dir/total-times-ms.txt"
awk '
  { values[NR] = $1; sum += $1 }
  END {
    if (!NR) exit 1
    median = NR % 2 ? values[(NR + 1) / 2] : (values[NR / 2] + values[NR / 2 + 1]) / 2
    printf "samples=%d\nmedian_ms=%.1f\nmean_ms=%.1f\nmin_ms=%d\nmax_ms=%d\n", NR, median, sum / NR, values[1], values[NR]
  }
' "$profile_dir/total-times-ms.txt" | tee "$profile_dir/summary.txt"

echo "Android profile saved in $profile_dir"
echo "Key files: summary.txt, cold-start-*.txt, memory.txt, cpu.txt, frames.txt, startup-log.txt"
