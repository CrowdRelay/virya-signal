#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
from pathlib import Path

# Keep ordinary feature growth tight. Deliberate compiler-profile changes (for
# example fat -> thin LTO) invalidate byte-for-byte comparability and establish
# a new baseline, while absolute raw + compressed ceilings remain enforced by
# check-web-dist.py in the same job.
RULES = {
    'wasmBytes': (1.03, 24 * 1024),
    'largestWasmBytes': (1.03, 24 * 1024),
    'codeBytes': (1.05, 32 * 1024),
    'codeGzipBytes': (1.03, 24 * 1024),
}


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument('current', type=Path)
    ap.add_argument('previous', type=Path)
    args = ap.parse_args()

    current = json.loads(args.current.read_text(encoding='utf-8'))
    previous = json.loads(args.previous.read_text(encoding='utf-8'))

    current_profile = current.get('buildProfile')
    previous_profile = previous.get('buildProfile')
    if not current_profile or not previous_profile or current_profile != previous_profile:
        print(
            'SIGNAL_WEB_REGRESSION=BASELINE_RESET '
            f"reason=build-profile-changed previous={previous_profile!r} current={current_profile!r}"
        )
        return

    bad: list[str] = []
    for key, (ratio, noise) in RULES.items():
        cur = float(current[key])
        prev = float(previous[key])
        limit = max(prev * ratio, prev + noise)
        if cur > limit:
            bad.append(f'{key}:{cur:g}>{limit:g}(prev={prev:g})')

    if bad:
        raise SystemExit('SIGNAL_WEB_REGRESSION=FAIL ' + ','.join(bad))
    print('SIGNAL_WEB_REGRESSION=PASS baseline=previous-successful-main')


if __name__ == '__main__':
    main()
