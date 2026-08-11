#!/bin/bash
set -euo pipefail

# Generate all icon variants from the canonical raster artwork.
# Keep the full tile and the transparent adaptive foreground separate so Android
# never renders a rounded app tile inside another launcher mask.
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT="$SCRIPT_DIR/.."
ICONS_DIR="$ROOT/src-tauri/icons"
LAUNCHER_DIR="$ROOT/src-tauri/launcher-assets/android"
FULL="$ICONS_DIR/virya-signal-brand-full.png"
FOREGROUND="$ICONS_DIR/virya-signal-brand-foreground.png"

[[ -f "$FULL" ]] || { echo "missing canonical full icon: $FULL" >&2; exit 1; }
[[ -f "$FOREGROUND" ]] || { echo "missing canonical adaptive foreground: $FOREGROUND" >&2; exit 1; }

resize() {
  local input="$1" output="$2" size="$3"
  mkdir -p "$(dirname "$output")"
  if command -v sips >/dev/null 2>&1; then
    sips -s format png -z "$size" "$size" "$input" --out "$output" >/dev/null
  elif command -v magick >/dev/null 2>&1; then
    magick "$input" -filter Lanczos -resize "${size}x${size}!" -strip "$output"
  else
    echo "Need macOS sips or ImageMagick to regenerate icons" >&2
    exit 1
  fi
  if command -v oxipng >/dev/null 2>&1; then
    oxipng -o 4 --strip all --nc "$output" >/dev/null
  fi
}

echo "=== Generating Virya Signal icons ==="

echo "[1/4] Master + desktop variants..."
resize "$FULL" "$ICONS_DIR/icon.png" 1024
for SIZE in 32 64 128 256 512; do
  resize "$FULL" "$ICONS_DIR/${SIZE}x${SIZE}.png" "$SIZE"
done
cp "$ICONS_DIR/256x256.png" "$ICONS_DIR/128x128@2x.png"

echo "[2/4] Windows Store logos..."
for SPEC in 30:Square30x30Logo.png 44:Square44x44Logo.png 71:Square71x71Logo.png 89:Square89x89Logo.png 107:Square107x107Logo.png 142:Square142x142Logo.png 150:Square150x150Logo.png 284:Square284x284Logo.png 310:Square310x310Logo.png 50:StoreLogo.png; do
  SIZE="${SPEC%%:*}"; NAME="${SPEC#*:}"
  resize "$FULL" "$ICONS_DIR/$NAME" "$SIZE"
done

echo "[3/4] Android launcher assets..."
resize "$FULL" "$LAUNCHER_DIR/play-store-512.png" 512
resize "$FOREGROUND" "$LAUNCHER_DIR/ic_launcher_foreground.png" 432
for SPEC in "mdpi:48:108" "hdpi:72:162" "xhdpi:96:216" "xxhdpi:144:324" "xxxhdpi:192:432"; do
  IFS=: read -r DENSITY LEGACY ADAPTIVE <<<"$SPEC"
  DIR="$LAUNCHER_DIR/mipmap-$DENSITY"
  resize "$FULL" "$DIR/ic_launcher.png" "$LEGACY"
  resize "$FULL" "$DIR/ic_launcher_round.png" "$LEGACY"
  resize "$FOREGROUND" "$DIR/ic_launcher_foreground.png" "$ADAPTIVE"
done

echo "[4/4] Desktop containers..."
if command -v magick >/dev/null 2>&1; then
  magick "$ICONS_DIR/32x32.png" "$ICONS_DIR/64x64.png" "$ICONS_DIR/128x128.png" "$ICONS_DIR/256x256.png" "$ICONS_DIR/icon.ico"
elif command -v convert >/dev/null 2>&1; then
  convert "$ICONS_DIR/32x32.png" "$ICONS_DIR/64x64.png" "$ICONS_DIR/128x128.png" "$ICONS_DIR/256x256.png" "$ICONS_DIR/icon.ico"
else
  echo "  [SKIP] .ico generation requires ImageMagick"
fi

if command -v iconutil >/dev/null 2>&1; then
  ICONSET="$ICONS_DIR/icon.iconset"
  rm -rf "$ICONSET"; mkdir -p "$ICONSET"
  for SPEC in 16:icon_16x16.png 32:icon_16x16@2x.png 32:icon_32x32.png 64:icon_32x32@2x.png 128:icon_128x128.png 256:icon_128x128@2x.png 256:icon_256x256.png 512:icon_256x256@2x.png 512:icon_512x512.png 1024:icon_512x512@2x.png; do
    SIZE="${SPEC%%:*}"; NAME="${SPEC#*:}"
    resize "$FULL" "$ICONSET/$NAME" "$SIZE"
  done
  iconutil -c icns "$ICONSET" -o "$ICONS_DIR/icon.icns"
  rm -rf "$ICONSET"
else
  echo "  [SKIP] .icns generation requires macOS iconutil"
fi

echo "=== Done! All icon variants generated from amber Signal Core artwork. ==="
