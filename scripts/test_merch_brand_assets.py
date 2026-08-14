#!/usr/bin/env python3
import struct
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def png_size(path: Path) -> tuple[int, int]:
    data = path.read_bytes()[:24]
    if len(data) < 24 or data[:8] != b"\x89PNG\r\n\x1a\n" or data[12:16] != b"IHDR":
        raise AssertionError(f"not a PNG: {path}")
    return struct.unpack(">II", data[16:24])


class MerchBrandAssetsTests(unittest.TestCase):
    def test_canonical_icons_remain_1024_square(self):
        for relative in (
            "src-tauri/icons/virya-signal-brand-full.png",
            "src-tauri/icons/virya-signal-brand-foreground.png",
        ):
            self.assertEqual(png_size(ROOT / relative), (1024, 1024))

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


if __name__ == "__main__":
    unittest.main()
