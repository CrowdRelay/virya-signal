#!/usr/bin/env python3
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
s=(ROOT/'src/app/types.rs').read_text()
assert '#[cfg(debug_assertions)]' in s
assert '#[cfg(not(debug_assertions))]' in s
assert 'option_env!("VIRYA_SIGNAL_E2E_API_BASE")' in s
assert 'const PRODUCTION_API_BASE: &str = "https://signal-api.virya.music/v1/";' in s
# The release branch must directly resolve to production and never consult the
# E2E environment variable at runtime.
release=s.split('#[cfg(not(debug_assertions))]',1)[1].split('pub(super) const POLICY_VERSION',1)[0]
assert 'PRODUCTION_API_BASE' in release and 'VIRYA_SIGNAL_E2E_API_BASE' not in release
bridge=(ROOT/'src/bridge/client.rs').read_text()
qr_block=bridge.split('pub async fn scan_qr()',1)[1].split('let value = scan_qr_js()',1)[0]
assert '#[cfg(debug_assertions)]' in qr_block
assert 'option_env!("VIRYA_SIGNAL_E2E_QR_PAYLOAD")' in qr_block
client=(ROOT/'src-tauri/src/api/client.rs').read_text()
assert 'option_env!("VIRYA_SIGNAL_E2E_STAFF_GATE_URL")' in client
release_gate=client.split('#[cfg(not(debug_assertions))]\nconst STAFF_GATE_URL',1)[1]
assert 'PRODUCTION_STAFF_GATE_URL' in release_gate
print('SIGNAL_E2E_BUILD_SAFETY=PASS debug_override=compile-time release=production-only')
