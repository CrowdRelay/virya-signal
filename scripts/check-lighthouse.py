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

PROFILE_LIMITS = {
    "mobile": {
        "scores": {"performance": 0.60, "accessibility": 0.95, "best-practices": 0.90},
        "audits": {
            "first-contentful-paint": 2500.0,
            "largest-contentful-paint": 3000.0,
            "total-blocking-time": 600.0,
            "cumulative-layout-shift": 0.10,
            "speed-index": 4000.0,
        },
    },
    "desktop": {
        "scores": {"performance": 0.85, "accessibility": 0.95, "best-practices": 0.90},
        "audits": {
            "first-contentful-paint": 1500.0,
            "largest-contentful-paint": 2000.0,
            "total-blocking-time": 400.0,
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


def validate(profile: str, current: dict, baseline: dict | None) -> list[str]:
    failures: list[str] = []
    limits = PROFILE_LIMITS[profile]

    for name, floor in limits["scores"].items():
        value = current["scores"][name]
        if value < floor:
            failures.append(
                f"{profile} {name}: {fmt_score(value)} < hard floor {fmt_score(floor)}"
            )

    for name, ceiling in limits["audits"].items():
        value = current["audits"][name]
        if value > ceiling:
            failures.append(
                f"{profile} {name}: {fmt_metric(name, value)} > hard ceiling {fmt_metric(name, ceiling)}"
            )

    if not baseline:
        return failures

    for name, value in current["scores"].items():
        old = baseline.get("scores", {}).get(name)
        if isinstance(old, (int, float)) and value < float(old) - SCORE_REGRESSION:
            failures.append(
                f"{profile} {name}: {fmt_score(value)} regressed from {fmt_score(float(old))} by >4 points"
            )

    for name, value in current["audits"].items():
        old = baseline.get("audits", {}).get(name)
        if not isinstance(old, (int, float)):
            continue
        old = float(old)
        if name == "cumulative-layout-shift":
            if value > old + CLS_REGRESSION:
                failures.append(
                    f"{profile} CLS: {value:.3f} regressed from {old:.3f} by >{CLS_REGRESSION:.2f}"
                )
            continue
        if value > old * METRIC_RATIO_REGRESSION and value > old + METRIC_ABSOLUTE_REGRESSION_MS:
            failures.append(
                f"{profile} {name}: {fmt_metric(name, value)} regressed from {fmt_metric(name, old)} by >35% and >500 ms"
            )

    return failures


def markdown(summary: dict, baseline_used: bool, failures: list[str]) -> str:
    lines = [
        "# Virya Signal · Lighthouse Watch",
        "",
        "Target: **100 / 100 / 100**. Hard CI floors are intentionally lower so Lighthouse noise does not block normal development.",
        "Absolute paint/layout ceilings use Chrome's observed trace metrics; the performance category remains a coarse Lantern safety signal.",
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
    if failures:
        lines += ["", "## Regressions", ""] + [f"- ❌ {failure}" for failure in failures]
    else:
        lines += ["", "**SIGNAL_LIGHTHOUSE=PASS**"]
    return "\n".join(lines) + "\n"


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--mobile", type=Path, required=True)
    parser.add_argument("--desktop", type=Path, required=True)
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
    compatible_baselines = 0
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
        failures.extend(validate(profile, summary[profile], old))

    args.json_out.parent.mkdir(parents=True, exist_ok=True)
    args.json_out.write_text(json.dumps(summary, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    baseline_used = compatible_baselines == len(summary)
    args.markdown_out.write_text(markdown(summary, baseline_used, failures), encoding="utf-8")

    print(args.markdown_out.read_text(encoding="utf-8"), end="")
    if failures:
        print("SIGNAL_LIGHTHOUSE=FAIL")
        return 1
    print("SIGNAL_LIGHTHOUSE=PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
