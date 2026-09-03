#!/usr/bin/env python3
"""Generate tenant-branded app icons from the Signal logo SVG + tenant palette.

Reads a tenant config JSON (from fetch-tenant-config.py) and produces PNG
icons at all the sizes Tauri requires. The base Signal logo is recolored
with the tenant's palette: the signal bars use the accent color, the
background uses the surface color.

If the tenant has no branding palette, the default Virya Signal colors are
used — the output is visually identical to the existing Virya icons.

Requires Pillow (PIL). Install: pip install Pillow

Usage:
  python3 scripts/generate-tenant-icons.py \
      --config tenant-config.json \
      --output-dir src-tauri/icons/tenant
"""
from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

try:
    from PIL import Image, ImageDraw
except ImportError:
    print("ERROR: Pillow is required. Install with: pip install Pillow", file=sys.stderr)
    raise SystemExit(1)


SIZES = [
    (32, 32, "32x32.png"),
    (128, 128, "128x128.png"),
    (256, 128, "128x128@2x.png"),
    (256, 256, "256x256.png"),
    (512, 512, "512x512.png"),
]

DEFAULT_PALETTE = {
    "primary": "#8b5cf6",
    "primaryContrast": "#ffffff",
    "accent": "#22d3ee",
    "surface": "#070908",
    "surfaceElevated": "#15171c",
    "text": "#f7f7f8",
    "textMuted": "#9ca3af",
    "success": "#22c55e",
    "warning": "#f59e0b",
    "danger": "#ef4444",
}


def hex_to_rgb(hex_color: str) -> tuple[int, int, int]:
    hex_color = hex_color.lstrip("#")
    return tuple(int(hex_color[i : i + 2], 16) for i in (0, 2, 4))


def draw_signal_icon(size: int, palette: dict[str, str]) -> Image.Image:
    """Draw a simplified Signal logo: three bars on a rounded background."""
    surface = hex_to_rgb(palette.get("surface", "#070908"))
    accent = hex_to_rgb(palette.get("accent", "#22d3ee"))

    img = Image.new("RGBA", (size, size), surface)
    draw = ImageDraw.Draw(img)

    # Three vertical bars, left-to-right, increasing height
    bar_width = max(1, size // 10)
    gap = max(1, size // 20)
    start_x = size // 2 - bar_width * 3 // 2 - gap
    heights = [size * 26 // 64, size * 36 // 64, size * 46 // 64]
    bar_y_bottom = size * 50 // 64

    for i, h in enumerate(heights):
        x = start_x + i * (bar_width + gap)
        y_top = bar_y_bottom - h
        radius = max(1, bar_width // 7)
        draw.rounded_rectangle(
            [x, y_top, x + bar_width, bar_y_bottom],
            radius=radius,
            fill=accent,
        )

    return img


def generate_icons(config: dict, output_dir: Path) -> None:
    palette = config.get("brandingPalette") or DEFAULT_PALETTE
    output_dir.mkdir(parents=True, exist_ok=True)

    for width, height, filename in SIZES:
        img = draw_signal_icon(max(width, height), palette)
        if width != height:
            img = img.resize((width, height), Image.LANCZOS)
        img.save(output_dir / filename, "PNG")
        print(f"Generated {filename} ({width}x{height})", file=sys.stderr)

    # ICO and ICNS (Tauri expects these for desktop builds, but Android only needs PNGs)
    # Generate a simple ICO from the 256x256
    icon_256 = draw_signal_icon(256, palette)
    icon_256.save(output_dir / "icon.ico", format="ICO", sizes=[(32, 32), (128, 128), (256, 256)])
    print("Generated icon.ico", file=sys.stderr)

    # ICNS is macOS-only; skip on non-Mac or if not needed
    try:
        icon_512 = draw_signal_icon(512, palette)
        icon_512.save(output_dir / "icon.icns", format="ICNS")
        print("Generated icon.icns", file=sys.stderr)
    except Exception:
        print("SKIP: could not generate icon.icns (non-fatal)", file=sys.stderr)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--config", required=True, help="Path to tenant config JSON")
    parser.add_argument("--output-dir", default="src-tauri/icons/tenant", help="Output directory for icons")
    args = parser.parse_args()

    config = json.loads(Path(args.config).read_text(encoding="utf-8"))
    output_dir = Path(args.output_dir)
    generate_icons(config, output_dir)
    print(f"Icons written to {output_dir}", file=sys.stderr)


if __name__ == "__main__":
    main()
