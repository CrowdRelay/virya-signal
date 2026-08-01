#!/usr/bin/env python3
from pathlib import Path
import json
import re
try:
    import tomllib
except ModuleNotFoundError:
    import tomli as tomllib

root = Path(__file__).resolve().parents[1]

for path in root.rglob('*.json'):
    json.loads(path.read_text())
for path in root.rglob('*.toml'):
    tomllib.loads(path.read_text())

required = [
    'src-tauri/src/lib.rs', 'src-tauri/src/api.rs', 'src-tauri/src/vault.rs',
    'src/app.rs', 'src/bridge.rs', 'src-tauri/capabilities/mobile.json',
    '.github/workflows/check.yml', '.github/workflows/mobile-smoke.yml',
    'rust-toolchain.toml', 'scripts/collect-mobile-artifact.py', 'boot.js',
]
for item in required:
    if not (root / item).is_file():
        raise SystemExit(f'missing {item}')

ui = (root / 'src/app.rs').read_text()
native = (root / 'src-tauri/src/lib.rs').read_text()
api = (root / 'src-tauri/src/api.rs').read_text()
invoked = set(re.findall(
    r'bridge::invoke(?:::<.*?>)?\(\s*"([a-z_]+)"', ui, re.S
))
invoked.update(re.findall(
    r'bridge::invoke_unit\(\s*"([a-z_]+)"', ui, re.S
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
    'staff/event-qr/overview', 'admin/event-qr/overview',
    'admin/admission/passes', 'me/referral', 'me/events?limit=50',
    'public/ticket-orders/{order_id}/wallet',
]
for path in required_paths:
    if path not in api:
        raise SystemExit(f'missing API contract path: {path}')

index = (root / 'index.html').read_text()
if 'data-trunk rel="copy-dir" href="public"' in index:
    raise SystemExit('index.html references the optional public directory')
for wasm_feature in ['--enable-bulk-memory', '--enable-bulk-memory-opt', '--enable-nontrapping-float-to-int']:
    if wasm_feature not in index:
        raise SystemExit(f'Rust 1.97 wasm-opt compatibility flag is missing: {wasm_feature}')

toolchain = tomllib.loads((root / 'rust-toolchain.toml').read_text())
if toolchain.get('toolchain', {}).get('channel') != '1.97.0':
    raise SystemExit('rust-toolchain.toml must pin Rust 1.97.0')

smoke = (root / '.github/workflows/mobile-smoke.yml').read_text()
for trigger_path in ['index.html', 'boot.js', 'Trunk.toml', 'styles.css', 'rust-toolchain.toml']:
    if f'- "{trigger_path}"' not in smoke:
        raise SystemExit(f'Android smoke workflow does not watch {trigger_path}')
if '--target aarch64' not in smoke or '--max-size-mib' not in smoke:
    raise SystemExit('Android smoke must build a bounded ARM64 APK')
if 'virya-signal-debug.apk' not in smoke:
    raise SystemExit('Android smoke artifact must be named Virya Signal')

signed_apk = (root / '.github/workflows/android-release-apk.yml').read_text()
if '--target aarch64' not in signed_apk or '--max-size-mib 150' not in signed_apk:
    raise SystemExit('Signed Android APK must be a bounded ARM64 build')

workflows = '\n'.join(path.read_text() for path in (root / '.github/workflows').glob('*.yml'))
if 'cargo install trunk' in workflows or 'cargo install tauri-cli' in workflows:
    raise SystemExit('workflow compiles a Tauri tool instead of installing a cached binary')
if 'toolchain: 1.88.0' in workflows or "tauri-cli --version '^2'" in workflows:
    raise SystemExit('workflow contains an obsolete or floating tool version')

native_manifest = tomllib.loads((root / 'src-tauri/Cargo.toml').read_text())
ui_manifest = tomllib.loads((root / 'Cargo.toml').read_text())
if native_manifest.get('package', {}).get('rust-version') != '1.97' or ui_manifest.get('package', {}).get('rust-version') != '1.97':
    raise SystemExit('Cargo manifests must require Rust 1.97')
if any(key.startswith('profile') for key in native_manifest):
    raise SystemExit('Cargo profiles must be declared at the workspace root')

print(f'static configuration and IPC contract check: OK ({len(invoked)} used / {len(registered)} registered commands)')
