import hashlib
import re
import struct
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
APP = (ROOT / "src/app.rs").read_text(encoding="utf-8")
STYLES = (ROOT / "styles.css").read_text(encoding="utf-8")
APPROVED_ICON_SHA256 = "a8db1cdc69decb1e1cf1774a484cb02b8101a40da191d9374613060a663fce3a"

ARGS_PATTERN = re.compile(
    r'#\[derive\(Serialize\)\]\n'
    r'(?P<attrs>(?:#\[[^\n]+\]\n)*)'
    r'struct\s+(?P<name>\w+Args)(?:<[^>]+>)?\s*\{'
)


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


class IpcAndBrandContractTests(unittest.TestCase):
    def test_every_top_level_ipc_args_wrapper_serializes_camel_case(self):
        found = {
            match.group("name"): match.group("attrs")
            for match in ARGS_PATTERN.finditer(APP)
        }
        expected = {
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
        self.assertNotIn("ApiArgs", found)
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

    def test_android_icon_uses_exact_approved_1024_source(self):
        icon = ROOT / "src-tauri/icons/icon.png"
        brand = ROOT / "src-tauri/icons/virya-signal-brand-full.png"
        raw = icon.read_bytes()
        self.assertEqual(raw[:8], b"\x89PNG\r\n\x1a\n")
        width, height = struct.unpack(">II", raw[16:24])
        self.assertEqual((width, height), (1024, 1024))
        self.assertEqual(sha256(icon), APPROVED_ICON_SHA256)
        self.assertEqual(sha256(brand), APPROVED_ICON_SHA256)
        self.assertEqual(icon.read_bytes(), brand.read_bytes())


if __name__ == "__main__":
    unittest.main()
