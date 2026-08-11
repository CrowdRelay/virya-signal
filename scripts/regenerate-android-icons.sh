#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
FULL="$ROOT/src-tauri/icons/virya-signal-brand-full.png"
FOREGROUND="$ROOT/src-tauri/icons/virya-signal-brand-foreground.png"
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
}

ASSETS="$ROOT/src-tauri/launcher-assets/android"
resize "$FULL" 512 "$ASSETS/play-store-512.png"
resize "$FOREGROUND" 432 "$ASSETS/ic_launcher_foreground.png"
for spec in "mdpi:48:108" "hdpi:72:162" "xhdpi:96:216" "xxhdpi:144:324" "xxxhdpi:192:432"; do
  IFS=: read -r density legacy foreground <<<"$spec"
  dir="$ASSETS/mipmap-$density"
  resize "$FULL" "$legacy" "$dir/ic_launcher.png"
  resize "$FULL" "$legacy" "$dir/ic_launcher_round.png"
  resize "$FOREGROUND" "$foreground" "$dir/ic_launcher_foreground.png"
done

# Refresh an already initialized Tauri Android project as well.
GEN="$ROOT/src-tauri/gen/android/app/src/main/res"
if [[ -d "$GEN" ]]; then
  for density in mdpi hdpi xhdpi xxhdpi xxxhdpi; do
    src="$ASSETS/mipmap-$density"
    dst="$GEN/mipmap-$density"
    mkdir -p "$dst"
    cp "$src/ic_launcher.png" "$dst/ic_launcher.png"
    cp "$src/ic_launcher_round.png" "$dst/ic_launcher_round.png"
    cp "$src/ic_launcher_foreground.png" "$dst/ic_launcher_foreground.png"
  done
  rm -rf "$ROOT/src-tauri/gen/android/app/build"
fi
printf 'Android launcher assets regenerated from amber Signal Core artwork\n'
