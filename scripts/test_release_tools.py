from __future__ import annotations

import importlib.util
import os
import unittest
from pathlib import Path
from unittest import mock


SCRIPT = Path(__file__).with_name("set-release-version.py")
SPEC = importlib.util.spec_from_file_location("set_release_version", SCRIPT)
assert SPEC and SPEC.loader
MODULE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MODULE)


class ReleaseVersionTests(unittest.TestCase):
    def test_normalizes_supported_tag_prefixes(self) -> None:
        self.assertEqual(MODULE.normalize_version("v0.2.1"), "0.2.1")
        self.assertEqual(MODULE.normalize_version("apk-v0.2.1"), "0.2.1")

    def test_auto_version_code_matches_play_run_number_scheme(self) -> None:
        with mock.patch.dict(os.environ, {"GITHUB_RUN_NUMBER": "321"}):
            self.assertEqual(MODULE.derive_android_version_code(), 100_000_321)

    def test_auto_version_code_fails_closed_without_a_run_number(self) -> None:
        with mock.patch.dict(os.environ, {}, clear=True):
            with self.assertRaises(ValueError):
                MODULE.derive_android_version_code()
        with mock.patch.dict(os.environ, {"GITHUB_RUN_NUMBER": "not-a-number"}):
            with self.assertRaises(ValueError):
                MODULE.derive_android_version_code()


if __name__ == "__main__":
    unittest.main()
