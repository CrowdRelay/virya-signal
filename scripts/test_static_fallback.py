import os
import subprocess
import sys
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]


class StaticFallbackTests(unittest.TestCase):
    def test_static_check_needs_no_third_party_toml_module(self):
        environment = os.environ.copy()
        environment["VIRYA_FORCE_TOML_FALLBACK"] = "1"
        generated_root = ROOT / "target"
        generated_json = generated_root / "static-check-fixture" / "events.json"
        generated_json.parent.mkdir(parents=True, exist_ok=True)
        generated_json.write_text('{"event":1}\n{"event":2}\n')
        try:
            result = subprocess.run(
                [sys.executable, str(ROOT / "scripts" / "static-check.py")],
                cwd=ROOT,
                env=environment,
                text=True,
                capture_output=True,
                check=False,
            )
        finally:
            generated_json.unlink(missing_ok=True)
            generated_json.parent.rmdir()
            # The fixture deliberately lives under ignored generated output;
            # remove the build root as well so source-only tests are read-only.
            generated_root.rmdir()
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("static configuration", result.stdout)


if __name__ == "__main__":
    unittest.main()
