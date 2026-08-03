#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SOURCE="$ROOT/src-tauri/icons/virya-signal-brand-full.png"
[[ -f "$SOURCE" ]] || SOURCE="$ROOT/src-tauri/icons/icon.png"
[[ -f "$SOURCE" ]] || { echo "missing canonical icon source" >&2; exit 1; }

resize() {
  local size="$1" destination="$2"
  mkdir -p "$(dirname "$destination")"
  if command -v sips >/dev/null 2>&1; then
    sips -s format png -z "$size" "$size" "$SOURCE" --out "$destination" >/dev/null
  elif command -v magick >/dev/null 2>&1; then
    magick "$SOURCE" -resize "${size}x${size}!" "$destination"
  else
    echo "Need macOS sips or ImageMagick to regenerate icons" >&2
    exit 1
  fi
}

ASSETS="$ROOT/src-tauri/launcher-assets/android"
resize 512 "$ASSETS/play-store-512.png"
resize 432 "$ASSETS/ic_launcher_foreground.png"
for spec in "mdpi:48:108" "hdpi:72:162" "xhdpi:96:216" "xxhdpi:144:324" "xxxhdpi:192:432"; do
  IFS=: read -r density legacy foreground <<<"$spec"
  dir="$ASSETS/mipmap-$density"
  resize "$legacy" "$dir/ic_launcher.png"
  resize "$legacy" "$dir/ic_launcher_round.png"
  resize "$foreground" "$dir/ic_launcher_foreground.png"
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
printf 'Android launcher assets regenerated from %s\n' "$SOURCE"
