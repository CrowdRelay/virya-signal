from __future__ import annotations

import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
UPLOAD_ARTIFACT_V7_SHA = "043fb46d1a93c77aae656e7c1c64a875d1fc6a0a"
RETIRED_UPLOAD_ARTIFACT_V4_SHA = "ea165f8d65b6e75b540449e92b4886f43607fa02"


class CiWebMetricsContracts(unittest.TestCase):
    def test_upload_artifact_is_node24_generation_everywhere(self) -> None:
        workflows = sorted((ROOT / ".github" / "workflows").glob("*.yml"))
        combined = "\n".join(path.read_text(encoding="utf-8") for path in workflows)
        self.assertNotIn(RETIRED_UPLOAD_ARTIFACT_V4_SHA, combined)
        upload_refs = [
            line.strip()
            for line in combined.splitlines()
            if "actions/upload-artifact@" in line
        ]
        self.assertTrue(upload_refs)
        for ref in upload_refs:
            self.assertIn(UPLOAD_ARTIFACT_V7_SHA, ref, ref)

    def test_web_metrics_upload_requires_generated_artifact(self) -> None:
        check = (ROOT / ".github" / "workflows" / "check.yml").read_text(encoding="utf-8")
        self.assertIn("id: web_metrics", check)
        self.assertIn("test -s artifacts/web-metrics.json", check)
        self.assertIn('echo "generated=true" >> "$GITHUB_OUTPUT"', check)
        self.assertIn(
            "if: ${{ always() && steps.web_metrics.outputs.generated == 'true' }}",
            check,
        )
        self.assertIn("if-no-files-found: error", check)


if __name__ == "__main__":
    unittest.main()
