#!/usr/bin/env python3
from __future__ import annotations

import argparse
import gzip
import json
import tomllib
from pathlib import Path

CODE = {'.wasm', '.js', '.mjs', '.css'}
ROOT = Path(__file__).resolve().parents[1]


def build_profile() -> str:
    manifest = tomllib.loads((ROOT / 'Cargo.toml').read_text(encoding='utf-8'))
    release = manifest.get('profile', {}).get('release', {})
    package = release.get('package', {}).get('virya-signal-ui', {})
    return ';'.join(
        [
            f"release.opt={release.get('opt-level')}",
            f"release.lto={release.get('lto')}",
            f"release.cgu={release.get('codegen-units')}",
            f"ui.opt={package.get('opt-level')}",
            f"ui.cgu={package.get('codegen-units')}",
        ]
    )


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument('root', type=Path)
    ap.add_argument('output', type=Path)
    args = ap.parse_args()

    files = [path for path in args.root.rglob('*') if path.is_file()]
    wasm = [path for path in files if path.suffix == '.wasm']
    code = [path for path in files if path.suffix.lower() in CODE]
    if not wasm:
        raise SystemExit('Signal WASM artifact missing')

    data = {
        'schema': 2,
        'buildProfile': build_profile(),
        'fileCount': len(files),
        'wasmBytes': sum(path.stat().st_size for path in wasm),
        'largestWasmBytes': max(path.stat().st_size for path in wasm),
        'codeBytes': sum(path.stat().st_size for path in code),
        'codeGzipBytes': sum(
            len(gzip.compress(path.read_bytes(), compresslevel=9, mtime=0))
            for path in code
        ),
    }
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(data, indent=2, sort_keys=True) + '\n', encoding='utf-8')
    print(
        'SIGNAL_WEB_METRICS=PASS '
        + ' '.join(f'{key}={value}' for key, value in data.items() if key != 'schema')
    )


if __name__ == '__main__':
    main()
