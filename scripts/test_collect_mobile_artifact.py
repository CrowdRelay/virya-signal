import hashlib
import importlib.util
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch


SCRIPT = Path(__file__).with_name("collect-mobile-artifact.py")
SPEC = importlib.util.spec_from_file_location("collect_mobile_artifact", SCRIPT)
MODULE = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
SPEC.loader.exec_module(MODULE)


class CollectMobileArtifactTests(unittest.TestCase):
    def test_hashes_file_without_loading_it_whole(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "virya-signal.apk"
            payload = b"virya-signal" * 100_000
            path.write_bytes(payload)
            self.assertEqual(MODULE.sha256(path), hashlib.sha256(payload).hexdigest())

    def test_prefers_single_universal_apk_and_ignores_test_outputs(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            universal = root / "release" / "app-universal-release.apk"
            universal.parent.mkdir(parents=True)
            universal.write_bytes(b"release")
            (universal.parent / "app-arm64-release.apk").write_bytes(b"abi")
            test = root / "androidTest" / "app-debug-androidTest.apk"
            test.parent.mkdir(parents=True)
            test.write_bytes(b"test")
            with patch.dict(MODULE.SEARCH_ROOTS, {"apk": root}):
                self.assertEqual(MODULE.candidates("apk"), [universal])


if __name__ == "__main__":
    unittest.main()
