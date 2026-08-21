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


def report(*, score: float = 0.97, observed_lcp: float = 1_215.0) -> dict:
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
        failures, advisories = CHECKER.validate("mobile", current, None)
        self.assertEqual(failures, [])
        self.assertEqual(advisories, [])

    def test_observed_lcp_and_category_floor_still_fail_closed(self) -> None:
        slow = self.load(report(observed_lcp=3_100.0))
        failures, _ = CHECKER.validate("mobile", slow, None)
        self.assertTrue(any("largest-contentful-paint" in item for item in failures))

    def test_category_floor_is_an_honest_ninety_five(self) -> None:
        # 0.94 must fail and 0.95 must pass: the gate is the number it claims
        # to be, which is only defensible because the reported value is a
        # median of several runs rather than a single noisy sample.
        below = self.load(report(score=0.94))
        failures, _ = CHECKER.validate("mobile", below, None)
        self.assertTrue(any("performance" in item for item in failures))
        at_floor = self.load(report(score=0.95))
        failures, _ = CHECKER.validate("mobile", at_floor, None)
        self.assertEqual(failures, [])

    def test_baseline_drift_is_advisory_and_never_blocks(self) -> None:
        # A run that is comfortably inside every absolute gate must stay green
        # even when it is slower than the previous green run. Blocking on this
        # is what made the audit fail on runner contention rather than on code.
        # 0.95 clears the floor exactly, while sitting far enough below a
        # previous perfect run to trip the drift rule.
        current = self.load(report(score=0.95))
        baseline = {
            "metric_source": CHECKER.METRIC_SOURCE,
            "scores": {"performance": 1.0, "accessibility": 1.0, "best-practices": 1.0},
            "audits": dict(current["audits"]),
        }
        failures, advisories = CHECKER.validate("mobile", current, baseline)
        self.assertEqual(failures, [])
        self.assertTrue(advisories, "drift should still be reported")


if __name__ == "__main__":
    unittest.main()
