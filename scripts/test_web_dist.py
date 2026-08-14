import tempfile
import unittest
from pathlib import Path

import importlib.util


SCRIPT = Path(__file__).with_name("check-web-dist.py")
SPEC = importlib.util.spec_from_file_location("check_web_dist", SCRIPT)
MODULE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MODULE)


class WebDistTests(unittest.TestCase):
    def test_accepts_output_within_budget(self):
        with tempfile.TemporaryDirectory() as directory:
            dist = Path(directory)
            (dist / "app.wasm").write_bytes(b"w" * 1024)
            (dist / "app.css").write_bytes(b"c" * 256)
            wasm, total, files = MODULE.inspect(dist, 2, 2)
            self.assertEqual(wasm, 1024)
            self.assertEqual(total, 1280)
            self.assertEqual(len(files), 2)

    def test_rejects_oversized_wasm(self):
        with tempfile.TemporaryDirectory() as directory:
            dist = Path(directory)
            (dist / "app.wasm").write_bytes(b"w" * 2049)
            with self.assertRaisesRegex(ValueError, "WASM size"):
                MODULE.inspect(dist, 2, 4)

    def test_rejects_missing_wasm(self):
        with tempfile.TemporaryDirectory() as directory:
            dist = Path(directory)
            (dist / "app.css").write_text("body{}")
            with self.assertRaisesRegex(ValueError, "no WASM"):
                MODULE.inspect(dist, 2, 4)

    def test_warns_before_the_hard_wasm_budget(self):
        self.assertEqual(MODULE.wasm_budget_state(1399 * 1024, 1400, 1536), "target")
        self.assertEqual(MODULE.wasm_budget_state(1401 * 1024, 1400, 1536), "warning")
        self.assertEqual(MODULE.wasm_budget_state(1537 * 1024, 1400, 1536), "fail")


if __name__ == "__main__":
    unittest.main()
