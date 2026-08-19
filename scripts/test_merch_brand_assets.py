#!/usr/bin/env python3
import json
import struct
import unittest
import zlib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def png_header(path: Path) -> tuple[int, int, int]:
    data = path.read_bytes()[:26]
    if len(data) < 26 or data[:8] != b"\x89PNG\r\n\x1a\n" or data[12:16] != b"IHDR":
        raise AssertionError(f"not a PNG: {path}")
    width, height = struct.unpack(">II", data[16:24])
    return width, height, data[25]


def png_size(path: Path) -> tuple[int, int]:
    width, height, _color_type = png_header(path)
    return width, height


def png_rgba_alpha_extrema(path: Path) -> tuple[int, int]:
    data = path.read_bytes()
    width, height, color_type = png_header(path)
    if color_type != 6 or data[24] != 8 or data[28] != 0:
        raise AssertionError(f"expected non-interlaced 8-bit RGBA PNG: {path}")

    offset = 8
    compressed = bytearray()
    while offset + 12 <= len(data):
        length = struct.unpack(">I", data[offset:offset + 4])[0]
        kind = data[offset + 4:offset + 8]
        payload = data[offset + 8:offset + 8 + length]
        if kind == b"IDAT":
            compressed.extend(payload)
        offset += 12 + length
        if kind == b"IEND":
            break

    raw = zlib.decompress(bytes(compressed))
    stride = width * 4
    previous = bytearray(stride)
    pos = 0
    alpha_min, alpha_max = 255, 0
    for _ in range(height):
        filter_type = raw[pos]
        pos += 1
        row = bytearray(raw[pos:pos + stride])
        pos += stride
        for index in range(stride):
            left = row[index - 4] if index >= 4 else 0
            up = previous[index]
            up_left = previous[index - 4] if index >= 4 else 0
            if filter_type == 1:
                row[index] = (row[index] + left) & 0xFF
            elif filter_type == 2:
                row[index] = (row[index] + up) & 0xFF
            elif filter_type == 3:
                row[index] = (row[index] + ((left + up) // 2)) & 0xFF
            elif filter_type == 4:
                p = left + up - up_left
                pa, pb, pc = abs(p - left), abs(p - up), abs(p - up_left)
                predictor = left if pa <= pb and pa <= pc else (up if pb <= pc else up_left)
                row[index] = (row[index] + predictor) & 0xFF
            elif filter_type != 0:
                raise AssertionError(f"unsupported PNG filter {filter_type}: {path}")
        alphas = row[3::4]
        alpha_min = min(alpha_min, min(alphas))
        alpha_max = max(alpha_max, max(alphas))
        previous = row
    return alpha_min, alpha_max


class MerchBrandAssetsTests(unittest.TestCase):
    def test_canonical_icons_remain_1024_square(self):
        for relative in (
            "src-tauri/icons/virya-signal-brand-full.png",
            "src-tauri/icons/virya-signal-brand-foreground.png",
        ):
            self.assertEqual(png_size(ROOT / relative), (1024, 1024))

    def test_tauri_bundle_png_icons_are_rgba(self):
        config = json.loads((ROOT / "src-tauri" / "tauri.conf.json").read_text())
        for relative in config["bundle"]["icon"]:
            if not relative.endswith(".png"):
                continue
            path = ROOT / "src-tauri" / relative
            self.assertEqual(png_header(path)[2], 6, f"Tauri bundle icon must be RGBA: {relative}")

    def test_android_adaptive_foregrounds_are_transparent_rgba(self):
        for path in sorted((ROOT / "src-tauri" / "icons" / "android").glob("mipmap-*/ic_launcher_foreground.png")):
            self.assertEqual(png_header(path)[2], 6, f"adaptive foreground must be RGBA: {path}")
            lo, hi = png_rgba_alpha_extrema(path)
            self.assertEqual(lo, 0, f"adaptive foreground needs transparent pixels: {path}")
            # Lanczos/resampling may cap the brightest alpha at 254; requiring
            # byte-perfect 255 is not a useful launcher-quality contract.
            self.assertGreaterEqual(hi, 250, f"adaptive foreground needs effectively opaque logo pixels: {path}")
        self.assertFalse((ROOT / "src-tauri" / "launcher-assets").exists())

    def test_android_ci_preserves_canonical_adaptive_foreground(self):
        workflow = (ROOT / ".github" / "workflows" / "_android-build.yml").read_text()
        self.assertIn("cargo tauri android init --ci --skip-targets-install", workflow)
        # If Tauri regenerates generic platform icons, it must use the V2 source;
        # prepare-android then reinstalls the canonical adaptive Android assets.
        if "cargo tauri icon" in workflow:
            self.assertIn("cargo tauri icon branding/signal-v2.svg", workflow)
        prepare = (ROOT / "scripts" / "prepare-android.py").read_text()
        self.assertIn("_install_android_launcher_assets()", prepare)
        self.assertIn('ANDROID_ICON_SOURCE_DIR = root / "src-tauri" / "icons" / "android"', prepare)

    def test_stage_pack_preview_is_bundled_and_wired(self):
        preview = ROOT / "bundle-stage-pack.webp"
        data = preview.read_bytes()
        self.assertGreater(len(data), 20_000)
        self.assertEqual(data[:4], b"RIFF")
        self.assertEqual(data[8:12], b"WEBP")
        self.assertIn(
            'data-trunk rel="copy-file" href="bundle-stage-pack.webp"',
            (ROOT / "index.html").read_text(),
        )
        merch = (ROOT / "src/app/fan/merch.rs").read_text()
        self.assertIn('STAGE_PACK_PREVIEW_URL', merch)
        self.assertIn('bundle.slug == "bundle-stage-pack"', merch)
        self.assertIn('product.slug == "echoes"', merch)
        canonical = ROOT.parent / "virya" / "public" / "covers" / "echoes.webp"
        if canonical.exists():
            self.assertEqual(preview.read_bytes(), canonical.read_bytes())


if __name__ == "__main__":
    unittest.main()
