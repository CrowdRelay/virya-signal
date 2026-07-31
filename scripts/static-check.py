#!/usr/bin/env python3
from pathlib import Path
import json
import re
import tomllib

root = Path(__file__).resolve().parents[1]

for path in root.rglob('*.json'):
    json.loads(path.read_text())
for path in root.rglob('*.toml'):
    tomllib.loads(path.read_text())

required = [
    'src-tauri/src/lib.rs', 'src-tauri/src/api.rs', 'src-tauri/src/vault.rs',
    'src/app.rs', 'src/bridge.rs', 'src-tauri/capabilities/mobile.json',
    '.github/workflows/check.yml',
]
for item in required:
    if not (root / item).is_file():
        raise SystemExit(f'missing {item}')

ui = (root / 'src/app.rs').read_text()
native = (root / 'src-tauri/src/lib.rs').read_text()
api = (root / 'src-tauri/src/api.rs').read_text()
invoked = set(re.findall(
    r'bridge::invoke(?:_unit)?::<.*?>\(\s*"([a-z_]+)"', ui, re.S
))
registered_match = re.search(r'tauri::generate_handler!\[(.*?)\]', native, re.S)
if not registered_match:
    raise SystemExit('missing Tauri invoke handler')
registered = {x.strip() for x in registered_match.group(1).split(',') if x.strip()}
missing = invoked - registered
if missing:
    raise SystemExit(f'UI invokes unregistered commands: {sorted(missing)}')

required_paths = [
    'public/events?limit=50', 'staff/admission/redeem', 'staff/coupons/redeem',
    'staff/event-qr/overview', 'admin/admission/passes',
]
for path in required_paths:
    if path not in api:
        raise SystemExit(f'missing API contract path: {path}')

for rust in root.rglob('*.rs'):
    text = rust.read_text()
    # Lightweight truncation check; real syntax/type checking is performed by CI.
    for left, right in [('(', ')'), ('[', ']'), ('{', '}')]:
        if text.count(left) != text.count(right):
            raise SystemExit(f'unbalanced {left}{right} in {rust.relative_to(root)}')

print(f'static configuration and IPC contract check: OK ({len(invoked)} used / {len(registered)} registered commands)')
