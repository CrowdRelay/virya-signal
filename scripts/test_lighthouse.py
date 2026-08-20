import importlib.util
import json
import unittest
from pathlib import Path
from tempfile import TemporaryDirectory

SCRIPT = Path(__file__).with_name("check-lighthouse.py")
SPEC = importlib.util.spec_from_file_location("signal_check_lighthouse", SCRIPT)
assert SPEC and SPEC.loader
CHECKER = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(CHECKER)


def report(*, score: float = 0.63, observed_lcp: float = 1_215.0) -> dict:
    simulated = {
        "first-contentful-paint": 2_673.0,
        "largest-contentful-paint": 12_456.0,
        "total-blocking-time": 311.0,
        "cumulative-layout-shift": 0.0,
        "speed-index": 3_162.0,
    }
    audits = {key: {"numericValue": value} for key, value in simulated.items()}
    audits["metrics"] = {
        "details": {
            "items": [{
                "observedFirstContentfulPaint": 1_202.0,
                "observedLargestContentfulPaint": observed_lcp,
                "totalBlockingTime": 311.0,
                "observedCumulativeLayoutShift": 0.0,
                "observedSpeedIndex": 1_495.0,
            }]
        }
    }
    return {
        "categories": {
            "performance": {"score": score},
            "accessibility": {"score": 1.0},
            "best-practices": {"score": 0.96},
        },
        "audits": audits,
    }


class LighthouseCheckerTests(unittest.TestCase):
    def load(self, payload: dict) -> dict:
        with TemporaryDirectory() as directory:
            path = Path(directory) / "report.json"
            path.write_text(json.dumps(payload), encoding="utf-8")
            return CHECKER.load_report(path)

    def test_hard_ux_ceiling_uses_observed_trace_not_lantern_projection(self) -> None:
        current = self.load(report())
        self.assertEqual(current["metric_source"], "observed-trace-v1")
        self.assertEqual(current["audits"]["largest-contentful-paint"], 1_215.0)
        self.assertEqual(current["simulated_audits"]["largest-contentful-paint"], 12_456.0)
        self.assertEqual(CHECKER.validate("mobile", current, None), [])

    def test_observed_lcp_and_category_safety_floor_still_fail_closed(self) -> None:
        slow = self.load(report(observed_lcp=3_100.0))
        self.assertTrue(any("largest-contentful-paint" in item for item in CHECKER.validate("mobile", slow, None)))
        low_score = self.load(report(score=0.59))
        self.assertTrue(any("performance" in item for item in CHECKER.validate("mobile", low_score, None)))


if __name__ == "__main__":
    unittest.main()
