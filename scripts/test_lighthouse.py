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
        self.assertTrue(any("largest-contentful-paint" in message for _, message in failures))

    def test_category_floor_is_an_honest_ninety_five(self) -> None:
        # 0.94 must fail and 0.95 must pass: the gate is the number it claims
        # to be, which is only defensible because the reported value is a
        # median of several runs rather than a single noisy sample.
        below = self.load(report(score=0.94))
        failures, _ = CHECKER.validate("mobile", below, None)
        self.assertTrue(any("performance" in message for _, message in failures))
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


class MeasurementReliabilityTests(unittest.TestCase):
    """A build may only fail when the *site* is at fault.

    Observed on one commit with no frontend change: mobile TBT 80, 1555 and
    2917 ms across three runs of the same bundle, while desktop in the same job
    spread 6 ms. The median of that population is 2917 ms and it is not a fact
    about the site. What separates the two cases is reproducibility — a real
    regression is slow every time.
    """

    def run_set(self, tbts: list[float], benchmarks: list[float] | None = None) -> dict:
        return {
            "label": "mobile",
            "runs": len(tbts),
            "total_blocking_time_ms": tbts,
            "total_blocking_time_spread_ms": max(tbts) - min(tbts),
            "benchmark_index": benchmarks if benchmarks is not None else [2300.0] * len(tbts),
        }

    def test_a_reproducible_slowdown_is_trusted(self) -> None:
        # Every run slow, tight spread: the site really is slower and this must
        # still block.
        self.assertIsNone(
            CHECKER.measurement_doubt("mobile", self.run_set([2_850.0, 2_900.0, 2_950.0]))
        )

    def test_wildly_dispersed_runs_are_not_a_measurement(self) -> None:
        doubt = CHECKER.measurement_doubt("mobile", self.run_set([80.0, 1_555.0, 2_917.0]))
        self.assertIsNotNone(doubt)
        self.assertIn("disagree", doubt)

    def test_a_degraded_machine_is_not_a_measurement(self) -> None:
        # Uniformly slow runs on a box Chrome itself reports as slow. The spread
        # is calm, so only benchmarkIndex can catch this one.
        doubt = CHECKER.measurement_doubt(
            "mobile", self.run_set([880.0, 900.0, 920.0], [700.0, 720.0, 690.0])
        )
        self.assertIsNotNone(doubt)
        self.assertIn("benchmarkIndex", doubt)

    def test_a_missing_run_set_never_manufactures_doubt(self) -> None:
        # No sidecar means no evidence the runner misbehaved, so the budgets
        # are enforced exactly as before.
        self.assertIsNone(CHECKER.measurement_doubt("mobile", None))

    def test_only_cpu_sensitive_budgets_can_be_excused(self) -> None:
        # Contrast and markup do not get faster on a quiet runner, so an
        # accessibility floor must block even when the timing numbers are junk.
        self.assertIn("performance", CHECKER.CPU_SENSITIVE_SCORES)
        self.assertNotIn("accessibility", CHECKER.CPU_SENSITIVE_SCORES)
        self.assertNotIn("best-practices", CHECKER.CPU_SENSITIVE_SCORES)
        self.assertIn("total-blocking-time", CHECKER.CPU_SENSITIVE_AUDITS)
        self.assertIn("speed-index", CHECKER.CPU_SENSITIVE_AUDITS)
        self.assertNotIn("cumulative-layout-shift", CHECKER.CPU_SENSITIVE_AUDITS)

    def test_every_failure_is_tagged_with_whether_doubt_could_excuse_it(self) -> None:
        payload = report(score=0.50, observed_lcp=9_000.0)
        payload["categories"]["accessibility"]["score"] = 0.80
        current = LighthouseCheckerTests.load(self, payload)
        failures, _ = CHECKER.validate("mobile", current, None)
        tagged = {message.split(": ", 1)[0]: sensitive for sensitive, message in failures}
        self.assertTrue(tagged["mobile performance"])
        self.assertTrue(tagged["mobile largest-contentful-paint"])
        self.assertFalse(tagged["mobile accessibility"])


class WorkflowWiringTests(unittest.TestCase):
    def test_a_doubtful_run_is_never_stored_as_the_baseline(self) -> None:
        workflow = (
            Path(__file__).resolve().parents[1]
            / ".github/workflows/signal-web-lighthouse.yml"
        ).read_text(encoding="utf-8")
        self.assertIn("--mobile-runs artifacts/lighthouse-mobile.runs.json", workflow)
        self.assertIn("--desktop-runs artifacts/lighthouse-desktop.runs.json", workflow)
        self.assertIn("id: budget", workflow)
        self.assertIn(
            "if: success() && steps.budget.outputs.reliable == 'true'", workflow
        )


if __name__ == "__main__":
    unittest.main()
