#!/usr/bin/env python3
"""Validate Virya Signal browser Lighthouse reports without making CI flaky.

Lighthouse's category score uses simulated Lantern timings. That model charges the
full initial WASM dependency graph to a text-only LCP candidate even when Chrome's
trace painted it much earlier. Absolute UX ceilings therefore use observed trace
metrics; the category score remains a coarse safety floor and relative baseline.
"""
from __future__ import annotations

import argparse
import json
from pathlib import Path

# These are honest gates, not aspirational ones: the audit reports the median
# of several runs, so a number here means the site can actually hit it rather
# than that one lucky run did. 0.95 across the board with a real target of 100.
PROFILE_LIMITS = {
    "mobile": {
        "scores": {"performance": 0.95, "accessibility": 0.95, "best-practices": 0.95},
        "audits": {
            "first-contentful-paint": 2500.0,
            "largest-contentful-paint": 3000.0,
            # Measured median at a CI-equivalent CPU slowdown is ~118 ms after
            # splitting module instantiation from the first mount, so this
            # ceiling sits roughly 3x above the real number.
            "total-blocking-time": 400.0,
            "cumulative-layout-shift": 0.10,
            "speed-index": 4000.0,
        },
    },
    "desktop": {
        "scores": {"performance": 0.95, "accessibility": 0.95, "best-practices": 0.95},
        "audits": {
            "first-contentful-paint": 1500.0,
            "largest-contentful-paint": 2000.0,
            "total-blocking-time": 200.0,
            "cumulative-layout-shift": 0.10,
            "speed-index": 3000.0,
        },
    },
}

OBSERVED_METRICS = {
    "first-contentful-paint": "observedFirstContentfulPaint",
    "largest-contentful-paint": "observedLargestContentfulPaint",
    "total-blocking-time": "totalBlockingTime",
    "cumulative-layout-shift": "observedCumulativeLayoutShift",
    "speed-index": "observedSpeedIndex",
}

SCORE_REGRESSION = 0.04
METRIC_RATIO_REGRESSION = 1.35
METRIC_ABSOLUTE_REGRESSION_MS = 500.0
CLS_REGRESSION = 0.04
METRIC_SOURCE = "observed-trace-v1"

# Audits whose value is a function of how much CPU the runner had. A busy box
# moves all of these; none of them can be moved by markup or contrast changes.
CPU_SENSITIVE_AUDITS = frozenset(
    {
        "first-contentful-paint",
        "largest-contentful-paint",
        "total-blocking-time",
        "speed-index",
    }
)
CPU_SENSITIVE_SCORES = frozenset({"performance"})

# A measurement is not a fact about the site when the machine was degraded or
# when the runs disagree with each other. Observed on one commit: mobile TBT
# 80, 1555 and 2917 ms in three consecutive runs of the same bundle, while
# desktop in the same job spread 6 ms. A genuine regression is reproducible —
# every run is slow and the spread stays small — so dispersion is what
# separates the two, and Chrome's own benchmarkIndex catches the case where
# the whole box is slow enough that even the spread looks calm.
TBT_SPREAD_CEILING_MS = 500.0
BENCHMARK_INDEX_FLOOR = 1000.0


def load_run_set(path: Path | None) -> dict | None:
    """The sidecar `lighthouse_median.py` writes beside the chosen report."""
    if path is None or not path.is_file():
        return None
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except (json.JSONDecodeError, OSError) as exc:
        print(f"SIGNAL_LIGHTHOUSE_RUNSET=IGNORED path={path} reason={exc}")
        return None


def measurement_doubt(profile: str, run_set: dict | None) -> str | None:
    """Why this profile's numbers cannot be trusted, or `None` if they can."""
    if not isinstance(run_set, dict):
        return None
    spread = run_set.get("total_blocking_time_spread_ms")
    if isinstance(spread, (int, float)) and float(spread) > TBT_SPREAD_CEILING_MS:
        values = run_set.get("total_blocking_time_ms")
        observed = (
            ", ".join(f"{float(value):.0f}" for value in values)
            if isinstance(values, list)
            else "unavailable"
        )
        return (
            f"{profile} runs disagree by {float(spread):.0f} ms of blocking time "
            f"(ceiling {TBT_SPREAD_CEILING_MS:.0f} ms; observed {observed} ms)"
        )
    indexes = run_set.get("benchmark_index")
    if isinstance(indexes, list) and indexes:
        slowest = min(float(value) for value in indexes if isinstance(value, (int, float)))
        if slowest < BENCHMARK_INDEX_FLOOR:
            return (
                f"{profile} ran on a degraded machine "
                f"(benchmarkIndex {slowest:.0f} < floor {BENCHMARK_INDEX_FLOOR:.0f})"
            )
    return None


def load_report(path: Path) -> dict:
    data = json.loads(path.read_text(encoding="utf-8"))
    categories = data.get("categories", {})
    audits = data.get("audits", {})
    scores: dict[str, float] = {}
    for name in ("performance", "accessibility", "best-practices"):
        score = categories.get(name, {}).get("score")
        if not isinstance(score, (int, float)):
            raise SystemExit(f"LIGHTHOUSE=FAIL {path}: missing category score {name}")
        scores[name] = float(score)

    metric_items = audits.get("metrics", {}).get("details", {}).get("items", [])
    observed = metric_items[0] if metric_items and isinstance(metric_items[0], dict) else {}
    values: dict[str, float] = {}
    simulated: dict[str, float] = {}
    for audit, observed_key in OBSERVED_METRICS.items():
        value = audits.get(audit, {}).get("numericValue")
        if not isinstance(value, (int, float)):
            raise SystemExit(f"LIGHTHOUSE=FAIL {path}: missing numeric audit {audit}")
        simulated[audit] = float(value)
        measured = observed.get(observed_key)
        values[audit] = float(measured) if isinstance(measured, (int, float)) else float(value)
    return {
        "metric_source": METRIC_SOURCE,
        "scores": scores,
        "audits": values,
        "simulated_audits": simulated,
    }


def fmt_score(score: float) -> str:
    return str(round(score * 100))


def fmt_metric(name: str, value: float) -> str:
    if name == "cumulative-layout-shift":
        return f"{value:.3f}"
    return f"{value:.0f} ms"


def validate(
    profile: str, current: dict, baseline: dict | None
) -> tuple[list[tuple[bool, str]], list[str]]:
    """Return (failures, advisory drift notes).

    Each failure is tagged with whether it is CPU-sensitive, because that is
    what decides whether a doubtful measurement is allowed to excuse it. An
    accessibility or best-practices floor is a fact about the markup and holds
    on any machine.

    Absolute floors and ceilings block: they say what the site must be able to
    do, and a median-of-N measurement can answer that honestly. Baseline
    comparison is advisory only. The baseline is whatever the last *successful*
    run recorded, so it drifts toward the luckiest measurement, and gating on
    it made green builds depend on how busy the runner was rather than on the
    code. The drift is still worth seeing, so it is reported, not enforced.
    """
    failures: list[tuple[bool, str]] = []
    advisories: list[str] = []
    limits = PROFILE_LIMITS[profile]

    for name, floor in limits["scores"].items():
        value = current["scores"][name]
        if value < floor:
            failures.append((
                name in CPU_SENSITIVE_SCORES,
                f"{profile} {name}: {fmt_score(value)} < hard floor {fmt_score(floor)}",
            ))

    for name, ceiling in limits["audits"].items():
        value = current["audits"][name]
        if value > ceiling:
            failures.append((
                name in CPU_SENSITIVE_AUDITS,
                f"{profile} {name}: {fmt_metric(name, value)} > hard ceiling "
                f"{fmt_metric(name, ceiling)}",
            ))

    if not baseline:
        return failures, advisories

    for name, value in current["scores"].items():
        old = baseline.get("scores", {}).get(name)
        if isinstance(old, (int, float)) and value < float(old) - SCORE_REGRESSION:
            advisories.append(
                f"{profile} {name}: {fmt_score(value)} drifted down from {fmt_score(float(old))}"
            )

    for name, value in current["audits"].items():
        old = baseline.get("audits", {}).get(name)
        if not isinstance(old, (int, float)):
            continue
        old = float(old)
        if name == "cumulative-layout-shift":
            if value > old + CLS_REGRESSION:
                advisories.append(
                    f"{profile} CLS: {value:.3f} drifted up from {old:.3f}"
                )
            continue
        if value > old * METRIC_RATIO_REGRESSION and value > old + METRIC_ABSOLUTE_REGRESSION_MS:
            advisories.append(
                f"{profile} {name}: {fmt_metric(name, value)} drifted up from {fmt_metric(name, old)}"
            )

    return failures, advisories


def markdown(
    summary: dict,
    baseline_used: bool,
    failures: list[str],
    advisories: list[str],
    doubts: list[str],
    excused: list[str],
) -> str:
    lines = [
        "# Virya Signal · Lighthouse Watch",
        "",
        "Target: **100 / 100 / 100**. CI blocks below **95** on every category.",
        "Each number is the **median of several runs**, so it reflects the site rather than how busy the runner was.",
        "Absolute paint/layout ceilings use Chrome's observed trace metrics. Baseline drift is reported but never blocks.",
        "",
        "| Profile | Performance | Accessibility | Best Practices | Observed FCP | Observed LCP | TBT | CLS | Observed Speed Index |",
        "|---|---:|---:|---:|---:|---:|---:|---:|---:|",
    ]
    for profile in ("mobile", "desktop"):
        data = summary[profile]
        s, a = data["scores"], data["audits"]
        lines.append(
            "| " + " | ".join(
                [
                    profile,
                    fmt_score(s["performance"]),
                    fmt_score(s["accessibility"]),
                    fmt_score(s["best-practices"]),
                    fmt_metric("first-contentful-paint", a["first-contentful-paint"]),
                    fmt_metric("largest-contentful-paint", a["largest-contentful-paint"]),
                    fmt_metric("total-blocking-time", a["total-blocking-time"]),
                    fmt_metric("cumulative-layout-shift", a["cumulative-layout-shift"]),
                    fmt_metric("speed-index", a["speed-index"]),
                ]
            ) + " |"
        )
    lines += ["", f"Previous successful baseline: **{'used' if baseline_used else 'not available yet'}**."]
    if doubts:
        lines += [
            "",
            "## Measurement was not trustworthy",
            "",
            "_The runner, not the site. These numbers are not recorded as a baseline._",
            "",
        ] + [f"- 🎲 {doubt}" for doubt in doubts]
    if excused:
        lines += [
            "",
            "## CPU-sensitive budgets not enforced on this run",
            "",
            "_Withheld because the measurement above is not trustworthy. "
            "Markup budgets were still enforced._",
            "",
        ] + [f"- ⏸️ {item}" for item in excused]
    if failures:
        lines += ["", "## Failures (blocking)", ""] + [f"- ❌ {failure}" for failure in failures]
    if advisories:
        lines += [
            "",
            "## Drift since the last successful run (advisory)",
            "",
            "_Not blocking. The baseline is the last green run, so it drifts toward the luckiest measurement._",
            "",
        ] + [f"- ⚠️ {advisory}" for advisory in advisories]
    if not failures:
        lines += ["", f"**SIGNAL_LIGHTHOUSE={'UNRELIABLE' if doubts else 'PASS'}**"]
    return "\n".join(lines) + "\n"


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--mobile", type=Path, required=True)
    parser.add_argument("--desktop", type=Path, required=True)
    parser.add_argument("--mobile-runs", type=Path,
                        help="sidecar run set written by lighthouse_median.py")
    parser.add_argument("--desktop-runs", type=Path,
                        help="sidecar run set written by lighthouse_median.py")
    parser.add_argument("--baseline", type=Path)
    parser.add_argument("--json-out", type=Path, required=True)
    parser.add_argument("--markdown-out", type=Path, required=True)
    args = parser.parse_args()

    summary = {
        "mobile": load_report(args.mobile),
        "desktop": load_report(args.desktop),
    }

    baseline = None
    if args.baseline and args.baseline.is_file():
        try:
            baseline = json.loads(args.baseline.read_text(encoding="utf-8"))
        except (json.JSONDecodeError, OSError) as exc:
            print(f"SIGNAL_LIGHTHOUSE_BASELINE=IGNORED reason={exc}")

    failures: list[str] = []
    excused: list[str] = []
    advisories: list[str] = []
    doubts: list[str] = []
    compatible_baselines = 0
    run_sets = {
        "mobile": load_run_set(args.mobile_runs),
        "desktop": load_run_set(args.desktop_runs),
    }
    for profile in ("mobile", "desktop"):
        old = baseline.get(profile) if isinstance(baseline, dict) else None
        if isinstance(old, dict) and old.get("metric_source") == METRIC_SOURCE:
            compatible_baselines += 1
        elif old is not None:
            print(
                f"SIGNAL_LIGHTHOUSE_BASELINE=IGNORED profile={profile} "
                "reason=metric-source-changed"
            )
            old = None
        profile_failures, profile_advisories = validate(profile, summary[profile], old)
        # A doubtful measurement withholds only the budgets that doubt can
        # explain. A performance ceiling missed on a starved box says nothing;
        # an accessibility floor missed on a starved box is still a real
        # accessibility failure, and it still blocks.
        doubt = measurement_doubt(profile, run_sets[profile])
        if doubt:
            doubts.append(doubt)
            print(f"SIGNAL_LIGHTHOUSE_MEASUREMENT=DOUBTFUL {doubt}")
        for cpu_sensitive, message in profile_failures:
            if doubt and cpu_sensitive:
                excused.append(message)
            else:
                failures.append(message)
        advisories.extend(profile_advisories)

    # The run set is part of the record: a summary that cannot say how it was
    # measured cannot be judged later either.
    for profile, run_set in run_sets.items():
        if isinstance(run_set, dict):
            summary[profile]["run_set"] = run_set
    reliable = not doubts
    summary["reliable"] = reliable

    args.json_out.parent.mkdir(parents=True, exist_ok=True)
    args.json_out.write_text(json.dumps(summary, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    baseline_used = compatible_baselines == 2
    args.markdown_out.write_text(
        markdown(summary, baseline_used, failures, advisories, doubts, excused), encoding="utf-8"
    )

    print(args.markdown_out.read_text(encoding="utf-8"), end="")
    if failures:
        print("SIGNAL_LIGHTHOUSE=FAIL")
        return 1
    if doubts:
        # Not a pass: nothing was proven about the site. Not a failure either,
        # because the only thing that went wrong was the machine. The workflow
        # reads `reliable` and refuses to store this as a baseline.
        print("SIGNAL_LIGHTHOUSE=UNRELIABLE")
        return 0
    print("SIGNAL_LIGHTHOUSE=PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
