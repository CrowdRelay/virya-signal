#!/usr/bin/env python3
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

ui_tenant = (ROOT / 'src/tenant.rs').read_text()
assert '#[cfg(debug_assertions)]' in ui_tenant
assert '#[cfg(not(debug_assertions))]' in ui_tenant
assert 'option_env!("VIRYA_SIGNAL_E2E_API_BASE")' in ui_tenant
assert 'const VIRYA_API_BASE: &str = "https://signal-api.virya.music/v1/";' in ui_tenant
ui_release = ui_tenant.split('#[cfg(not(debug_assertions))]', 1)[1].split('pub(crate) const DEFAULT_COUNTRY_CODE', 1)[0]
assert 'TENANT_API_BASE' in ui_release
assert 'VIRYA_SIGNAL_E2E_API_BASE' not in ui_release

bridge = (ROOT / 'src/bridge/client.rs').read_text()
qr_block = bridge.split('pub async fn scan_qr()', 1)[1].split('let value = scan_qr_js()', 1)[0]
assert '#[cfg(debug_assertions)]' in qr_block
assert 'option_env!("VIRYA_SIGNAL_E2E_QR_PAYLOAD")' in qr_block

native_tenant = (ROOT / 'src-tauri/src/tenant.rs').read_text()
assert '#[cfg(debug_assertions)]' in native_tenant
assert '#[cfg(not(debug_assertions))]' in native_tenant
assert 'option_env!("VIRYA_SIGNAL_E2E_API_BASE")' in native_tenant
assert 'option_env!("VIRYA_SIGNAL_E2E_STAFF_GATE_URL")' in native_tenant
assert 'option_env!("VIRYA_SIGNAL_E2E_STAFF_GATE_ORIGIN")' in native_tenant
api_release = native_tenant.split('#[cfg(not(debug_assertions))]\npub(crate) const API_BASE', 1)[1].split('pub(crate) const DEFAULT_COUNTRY_CODE', 1)[0]
assert 'TENANT_API_BASE' in api_release
assert 'VIRYA_SIGNAL_E2E_API_BASE' not in api_release
staff_release = native_tenant.split('#[cfg(not(debug_assertions))]\npub(crate) const STAFF_GATE_URL', 1)[1]
assert 'TENANT_STAFF_GATE_URL' in staff_release
assert 'VIRYA_SIGNAL_E2E_STAFF_GATE_URL' not in staff_release.split('#[cfg(debug_assertions)]', 1)[0]

native_api = (ROOT / 'src-tauri/src/api.rs').read_text()
assert '.post(crate::tenant::STAFF_GATE_URL)' in native_api
assert '.header(ORIGIN, crate::tenant::STAFF_GATE_ORIGIN)' in native_api

print('SIGNAL_E2E_BUILD_SAFETY=PASS debug_override=compile-time release=tenant-production-only')
