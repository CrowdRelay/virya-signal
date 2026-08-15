#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ICONS="$ROOT/src-tauri/icons"
ANDROID="$ICONS/android"
FULL="$ICONS/virya-signal-brand-full.png"
FOREGROUND="$ICONS/virya-signal-brand-foreground.png"
[[ -f "$FULL" ]] || { echo "missing canonical full icon" >&2; exit 1; }
[[ -f "$FOREGROUND" ]] || { echo "missing canonical adaptive foreground" >&2; exit 1; }

resize() {
  local source="$1" size="$2" destination="$3"
  mkdir -p "$(dirname "$destination")"
  if command -v sips >/dev/null 2>&1; then
    sips -s format png -z "$size" "$size" "$source" --out "$destination" >/dev/null
  elif command -v magick >/dev/null 2>&1; then
    magick "$source" -filter Lanczos -resize "${size}x${size}!" -strip "$destination"
  else
    echo "Need macOS sips or ImageMagick to regenerate icons" >&2
    exit 1
  fi
  python3 - "$destination" <<'PY_RGBA'
from pathlib import Path
import sys
p = Path(sys.argv[1])
data = p.read_bytes()[:26]
if len(data) < 26 or data[:8] != b"\x89PNG\r\n\x1a\n" or data[12:16] != b"IHDR":
    raise SystemExit(f"not a PNG: {p}")
if data[25] != 6:
    raise SystemExit(f"generated PNG is not RGBA: {p} (color_type={data[25]})")
PY_RGBA
}

for spec in "mdpi:48:108" "hdpi:72:162" "xhdpi:96:216" "xxhdpi:144:324" "xxxhdpi:192:432"; do
  IFS=: read -r density legacy foreground <<<"$spec"
  dir="$ANDROID/mipmap-$density"
  resize "$FULL" "$legacy" "$dir/ic_launcher.png"
  resize "$FULL" "$legacy" "$dir/ic_launcher_round.png"
  resize "$FOREGROUND" "$foreground" "$dir/ic_launcher_foreground.png"
done

mkdir -p "$ANDROID/mipmap-anydpi-v26" "$ANDROID/values"
cat > "$ANDROID/mipmap-anydpi-v26/ic_launcher.xml" <<'XML'
<?xml version="1.0" encoding="utf-8"?>
<adaptive-icon xmlns:android="http://schemas.android.com/apk/res/android">
  <foreground android:drawable="@mipmap/ic_launcher_foreground"/>
  <background android:drawable="@color/ic_launcher_background"/>
</adaptive-icon>
XML
cat > "$ANDROID/values/ic_launcher_background.xml" <<'XML'
<?xml version="1.0" encoding="utf-8"?>
<resources>
  <color name="ic_launcher_background">#080808</color>
</resources>
XML

# Refresh an already initialized Tauri Android project using the same canonical
# tracked resources that CI will stage in prepare-android.py.
GEN="$ROOT/src-tauri/gen/android/app/src/main/res"
if [[ -d "$GEN" ]]; then
  for subdir in mipmap-anydpi-v26 mipmap-mdpi mipmap-hdpi mipmap-xhdpi mipmap-xxhdpi mipmap-xxxhdpi values; do
    [[ -d "$ANDROID/$subdir" ]] || continue
    mkdir -p "$GEN/$subdir"
    cp "$ANDROID/$subdir"/* "$GEN/$subdir/"
  done
  rm -rf "$ROOT/src-tauri/gen/android/app/build"
fi
printf 'Android launcher assets regenerated in src-tauri/icons/android\n'
