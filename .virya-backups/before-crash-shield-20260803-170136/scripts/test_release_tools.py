from __future__ import annotations

import importlib.util
import unittest
from pathlib import Path


SCRIPT = Path(__file__).with_name("set-release-version.py")
SPEC = importlib.util.spec_from_file_location("set_release_version", SCRIPT)
assert SPEC and SPEC.loader
MODULE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MODULE)


class ReleaseVersionTests(unittest.TestCase):
    def test_normalizes_supported_tag_prefixes(self) -> None:
        self.assertEqual(MODULE.normalize_version("v0.2.1"), "0.2.1")
        self.assertEqual(MODULE.normalize_version("apk-v0.2.1"), "0.2.1")

    def test_derives_monotonic_android_code(self) -> None:
        self.assertEqual(MODULE.derive_android_version_code("0.2.0"), 2000)
        self.assertEqual(MODULE.derive_android_version_code("0.2.1"), 2001)
        self.assertEqual(MODULE.derive_android_version_code("1.0.0"), 1_000_000)

    def test_rejects_ambiguous_large_components(self) -> None:
        with self.assertRaises(ValueError):
            MODULE.derive_android_version_code("0.1000.0")


if __name__ == "__main__":
    unittest.main()
