#!/usr/bin/env python3
"""Pick the median run from repeated Lighthouse reports of one profile.

A single Lighthouse run on a shared CI runner is not a measurement of the
site, it is a measurement of the site plus whatever else the runner happened
to be doing. Observed spread on this project across commits that changed no
frontend code: mobile TBT 194-1200 ms, mobile FCP 136-495 ms. Gating on one
sample turns that spread into random build failures.

Lighthouse's own guidance is to run several times and report the median, which
is what this does: the run with the median performance score wins, ties broken
by median total blocking time so the representative run is stable rather than
whichever sorted first.

Median is deliberate rather than best-of-N. Best-of-N would report the
luckiest run and quietly ratchet the baseline toward numbers the site cannot
reproduce; median reports a run the site can actually hit again.

The median alone is not enough when the population is multi-modal. Observed on
one commit: mobile TBT 80, 1555, 2917 ms in three consecutive runs, while
desktop in the same job held a 6 ms spread. The median of that is 2917 ms and
it is not a fact about the site. So this also writes a sidecar record of the
whole run set — every TBT and every `benchmarkIndex` Chrome measured for the
machine — and `check-lighthouse.py` uses it to tell a reproducible regression
(consistently slow) from a starved runner (wildly dispersed).
"""

from __future__ import annotations

import argparse
import json
import shutil
import statistics
import sys
from pathlib import Path


def score_of(report: dict) -> float:
    categories = report.get("categories", {})
    performance = categories.get("performance", {}).get("score")
    if not isinstance(performance, (int, float)):
        raise SystemExit("LIGHTHOUSE_MEDIAN=FAIL report is missing a performance score")
    return float(performance)


def tbt_of(report: dict) -> float:
    value = report.get("audits", {}).get("total-blocking-time", {}).get("numericValue")
    return float(value) if isinstance(value, (int, float)) else 0.0


def benchmark_index_of(report: dict) -> float:
    """Chrome's own measurement of how fast the machine running the audit is.

    This is a property of the runner, not of the page, so it is the one number
    that can separate "the site got slower" from "the box was busy".
    """
    value = report.get("environment", {}).get("benchmarkIndex")
    return float(value) if isinstance(value, (int, float)) else 0.0


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--report", type=Path, action="append", required=True,
                        help="a Lighthouse JSON report; pass once per run")
    parser.add_argument("--out", type=Path, required=True,
                        help="path to copy the median report to")
    parser.add_argument("--label", default="profile")
    args = parser.parse_args()

    paths = [p for p in args.report if p.is_file()]
    if not paths:
        raise SystemExit("LIGHTHOUSE_MEDIAN=FAIL no readable reports were provided")

    runs = []
    for path in paths:
        try:
            report = json.loads(path.read_text(encoding="utf-8"))
        except (json.JSONDecodeError, OSError) as exc:
            raise SystemExit(f"LIGHTHOUSE_MEDIAN=FAIL cannot read {path}: {exc}") from exc
        runs.append((score_of(report), tbt_of(report), path, benchmark_index_of(report)))

    # Sort by score, then by TBT, and take the middle run. For an even count
    # this takes the lower-middle, which is the conservative side.
    runs.sort(key=lambda run: (run[0], -run[1]))
    chosen = runs[(len(runs) - 1) // 2]

    args.out.parent.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(chosen[2], args.out)

    # Keep the human-readable report of the same run, not of some other one.
    chosen_html = Path(str(chosen[2]).replace(".report.json", ".report.html"))
    if chosen_html.is_file():
        shutil.copyfile(chosen_html, Path(str(args.out).replace(".report.json", ".report.html")))

    tbt_values = [run[1] for run in runs]
    benchmark_values = [run[3] for run in runs]
    spread = max(tbt_values) - min(tbt_values)
    sidecar = Path(str(args.out).replace(".report.json", ".runs.json"))
    sidecar.write_text(
        json.dumps(
            {
                "label": args.label,
                "runs": len(runs),
                "total_blocking_time_ms": tbt_values,
                "total_blocking_time_spread_ms": spread,
                "benchmark_index": benchmark_values,
            },
            indent=2,
            sort_keys=True,
        )
        + "\n",
        encoding="utf-8",
    )

    scores = [f"{run[0] * 100:.0f}" for run in runs]
    tbts = [f"{run[1]:.0f}" for run in runs]
    print(
        f"LIGHTHOUSE_MEDIAN={args.label} runs={len(runs)} "
        f"scores=[{','.join(scores)}] tbt_ms=[{','.join(tbts)}] "
        f"chosen_score={chosen[0] * 100:.0f} chosen_tbt_ms={chosen[1]:.0f} "
        f"spread_tbt_ms={spread:.0f} "
        f"benchmark_index=[{','.join(f'{value:.0f}' for value in benchmark_values)}]"
    )
    if len(runs) > 1:
        print(f"LIGHTHOUSE_MEDIAN_STDEV_TBT_MS={statistics.pstdev([run[1] for run in runs]):.0f}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
