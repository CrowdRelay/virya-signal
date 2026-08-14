#!/usr/bin/env python3
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


class SharedOpsContract(unittest.TestCase):
    def test_ops_runtime_summaries_are_shared_between_web_and_native(self):
        shared = (ROOT / "crates/virya-signal-contracts/src/ops.rs").read_text()
        web = (ROOT / "src/models.rs").read_text()
        native = (ROOT / "src-tauri/src/models.rs").read_text()
        for name in (
            "QueueSummary",
            "DatabaseRuntimeSummary",
            "AreaRuntimeSummary",
            "HttpRequestSummary",
            "OpsSummary",
        ):
            self.assertIn(f"pub struct {name}", shared)
            self.assertNotIn(f"pub struct {name}", web)
            self.assertNotIn(f"pub struct {name}", native)
        self.assertIn("pub use virya_signal_contracts::ops::*;", web)
        self.assertIn("pub use virya_signal_contracts::ops::*;", native)


if __name__ == "__main__":
    unittest.main()
