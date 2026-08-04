#!/bin/bash
set -eo pipefail

# Generate all icon variants from the SVG source
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT="$SCRIPT_DIR/.."
ICONS_DIR="$ROOT/src-tauri/icons"
LAUNCHER_DIR="$ROOT/src-tauri/launcher-assets/android"
SVG_FULL="$ICONS_DIR/virya-signal.svg"
SVG_FG="$ICONS_DIR/virya-signal-foreground.svg"

render() {
  local input="$1" output="$2" size="$3"
  npx --yes resvg-cli "$input" "$output" --fit-width "$size" --fit-height "$size"
  if command -v oxipng &>/dev/null; then
    oxipng -o 4 --strip all --nc "$output"
  fi
}

echo "=== Generating Virya Signal icons ==="

# Generate the master 1024x1024 PNG
echo "[1/4] Rendering master 1024x1024 from SVG..."
render "$SVG_FULL" "$ICONS_DIR/icon.png" 1024

# Desktop/Tauri icon sizes
echo "[2/4] Generating desktop icon variants..."
for SIZE in 32 128 256 512; do
  render "$SVG_FULL" "$ICONS_DIR/${SIZE}x${SIZE}.png" "$SIZE"
done
# 128x128@2x is 256x256
cp "$ICONS_DIR/256x256.png" "$ICONS_DIR/128x128@2x.png"

# Windows Store logos
echo "[3/4] Generating Windows Store logos..."
render "$SVG_FULL" "$ICONS_DIR/Square30x30Logo.png" 30
render "$SVG_FULL" "$ICONS_DIR/Square44x44Logo.png" 44
render "$SVG_FULL" "$ICONS_DIR/Square71x71Logo.png" 71
render "$SVG_FULL" "$ICONS_DIR/Square89x89Logo.png" 89
render "$SVG_FULL" "$ICONS_DIR/Square107x107Logo.png" 107
render "$SVG_FULL" "$ICONS_DIR/Square142x142Logo.png" 142
render "$SVG_FULL" "$ICONS_DIR/Square150x150Logo.png" 150
render "$SVG_FULL" "$ICONS_DIR/Square284x284Logo.png" 284
render "$SVG_FULL" "$ICONS_DIR/Square310x310Logo.png" 310
render "$SVG_FULL" "$ICONS_DIR/StoreLogo.png" 50

# Android launcher icons
echo "[4/4] Generating Android launcher assets..."
render "$SVG_FULL" "$LAUNCHER_DIR/play-store-512.png" 512
render "$SVG_FULL" "$LAUNCHER_DIR/ic_launcher_foreground.png" 512

# Android mipmap densities
for PAIR in "mipmap-mdpi:108" "mipmap-hdpi:162" "mipmap-xhdpi:216" "mipmap-xxhdpi:324" "mipmap-xxxhdpi:432"; do
  DENSITY="${PAIR%%:*}"
  SIZE="${PAIR##*:}"
  DIR="$LAUNCHER_DIR/$DENSITY"
  mkdir -p "$DIR"
  render "$SVG_FG" "$DIR/ic_launcher_foreground.png" "$SIZE"
  render "$SVG_FULL" "$DIR/ic_launcher.png" "$SIZE"
  render "$SVG_FULL" "$DIR/ic_launcher_round.png" "$SIZE"
done

# Generate .ico (requires ImageMagick)
if command -v magick &>/dev/null; then
  magick "$ICONS_DIR/32x32.png" "$ICONS_DIR/128x128.png" "$ICONS_DIR/256x256.png" "$ICONS_DIR/icon.ico"
  echo "  ✓ icon.ico generated"
elif command -v convert &>/dev/null; then
  convert "$ICONS_DIR/32x32.png" "$ICONS_DIR/128x128.png" "$ICONS_DIR/256x256.png" "$ICONS_DIR/icon.ico"
  echo "  ✓ icon.ico generated"
else
  echo "  [SKIP] .ico generation requires ImageMagick"
fi

# Generate .icns for macOS
if command -v iconutil &>/dev/null; then
  ICONSET="$ICONS_DIR/icon.iconset"
  rm -rf "$ICONSET"
  mkdir -p "$ICONSET"
  render "$SVG_FULL" "$ICONSET/icon_16x16.png" 16
  render "$SVG_FULL" "$ICONSET/icon_16x16@2x.png" 32
  render "$SVG_FULL" "$ICONSET/icon_32x32.png" 32
  render "$SVG_FULL" "$ICONSET/icon_32x32@2x.png" 64
  render "$SVG_FULL" "$ICONSET/icon_128x128.png" 128
  render "$SVG_FULL" "$ICONSET/icon_128x128@2x.png" 256
  render "$SVG_FULL" "$ICONSET/icon_256x256.png" 256
  render "$SVG_FULL" "$ICONSET/icon_256x256@2x.png" 512
  render "$SVG_FULL" "$ICONSET/icon_512x512.png" 512
  render "$SVG_FULL" "$ICONSET/icon_512x512@2x.png" 1024
  iconutil -c icns "$ICONSET" -o "$ICONS_DIR/icon.icns"
  rm -rf "$ICONSET"
  echo "  ✓ icon.icns generated"
else
  echo "  [SKIP] .icns generation requires macOS iconutil"
fi

echo ""
echo "=== Done! All icon variants generated. ==="
