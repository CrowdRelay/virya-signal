import re
import struct
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
APP = (ROOT / "src/app.rs").read_text(encoding="utf-8")
STYLES = (ROOT / "styles.css").read_text(encoding="utf-8")

ARGS_PATTERN = re.compile(
    r'#\[derive\(Serialize\)\]\n'
    r'(?P<attrs>(?:#\[[^\n]+\]\n)*)'
    r'struct\s+(?P<name>\w+Args)(?:<[^>]+>)?\s*\{'
)


class IpcAndBrandContractTests(unittest.TestCase):
    def test_every_top_level_ipc_args_wrapper_serializes_camel_case(self):
        found = {
            match.group("name"): match.group("attrs")
            for match in ARGS_PATTERN.finditer(APP)
        }
        expected = {
            "ApiArgs",
            "EventArgs",
            "RedeemArgs",
            "CouponArgs",
            "ReferenceArgs",
            "CampaignIdArgs",
            "RetryArgs",
            "ClaimArgs",
            "ImportWalletArgs",
            "OrderArgs",
            "WalletQrArgs",
        }
        self.assertTrue(expected.issubset(found), expected - found.keys())
        for name, attrs in found.items():
            self.assertIn(
                '#[serde(rename_all = "camelCase")]',
                attrs,
                f"{name} would send snake_case top-level Tauri argument keys",
            )

    def test_launcher_uses_canonical_virya_signal_brand(self):
        self.assertIn("VIRYA SIGNAL", APP)
        self.assertNotIn("VIRYA MOBILE", APP)
        self.assertIn('class="signal-mark signal-logo"', APP)
        self.assertIn('aria-label="Logo Virya Signal"', APP)
        self.assertIn("@keyframes signal-logo-lock", STYLES)
        self.assertIn("animation-fill-mode: both", STYLES)

    def test_android_icon_uses_canonical_1024_source(self):
        icon = ROOT / "src-tauri/icons/icon.png"
        raw = icon.read_bytes()
        self.assertEqual(raw[:8], b"\x89PNG\r\n\x1a\n")
        width, height = struct.unpack(">II", raw[16:24])
        self.assertEqual((width, height), (1024, 1024))
        svg = (ROOT / "src-tauri/icons/virya-signal.svg").read_text()
        self.assertIn('id="signal-bars"', svg)
        signal_group = svg.split('id="signal-bars"', 1)[1].split("</g>", 1)[0]
        self.assertEqual(signal_group.count("<rect"), 3)


if __name__ == "__main__":
    unittest.main()
