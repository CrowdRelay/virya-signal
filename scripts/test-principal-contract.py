#!/usr/bin/env python3
"""Regression guards for the 0.4.2 max ecosystem pass."""
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
app = (ROOT / "src/app.rs").read_text(encoding="utf-8")
native = (ROOT / "src-tauri/src/lib.rs").read_text(encoding="utf-8")
models = (ROOT / "src/models.rs").read_text(encoding="utf-8")
native_models = (ROOT / "src-tauri/src/models.rs").read_text(encoding="utf-8")
api = (ROOT / "src-tauri/src/api.rs").read_text(encoding="utf-8")
config = (ROOT / "src-tauri/tauri.conf.json").read_text(encoding="utf-8")

assert '"version": "0.4.2"' in config
assert "FanTab::Game" in app
assert "fn FanGame(" in app
assert "fn open_area_game(" in app
assert "let open_game" not in app, "reusable click action must not become an FnOnce closure"
assert app.count('open_area_game(error)') == 2
assert '"fan_area_wallet"' in app
assert '"operator_ops_overview"' in app
assert '"operator_retry"' in app
assert 'fn fan_area_wallet(' in native
assert 'fn operator_ops_overview(' in native
assert 'fn operator_retry(' in native
assert native.count('area_wallet_id: uuid::Uuid::new_v4().to_string()') == 2, (
    "both signup and confirmation must provision a stable AREA wallet"
)
assert 'pub struct AreaWallet' in models and 'pub struct AreaWallet' in native_models
assert 'pub struct OperatorOpsOverview' in models and 'pub struct OperatorOpsOverview' in native_models
assert 'AREA_WALLET_URL' in api
assert 'AREA_COOKIE' in api
assert 'future::join3' in api and 'unavailable_sources' in api
assert 'futures_util::future::join3' in api, 'ops reads should execute concurrently'

# 0.4.2 offline Show Mode contract
assert '"show_mode_prepare"' in app
assert '"show_mode_scan"' in app
assert '"show_mode_sync"' in app
assert 'async fn show_mode_prepare(' in native
assert 'async fn show_mode_scan(' in native
assert 'async fn show_mode_sync(' in native
assert 'snapshot_is_active' in native
assert 'show_mode_checksum' in native
assert 'session.scans.len() >= 10_000' in native
assert 'Tryb offline obsługuje wyłącznie trwałe bilety t1' in native
assert 'qr_sha256' in native_models
assert 'SHOW_MODE_STORE_KEY' in (ROOT / "src-tauri/src/vault.rs").read_text(encoding="utf-8")
assert 'qr_token:' not in native_models[native_models.index('pub struct ShowModeQueuedScan'):], (
    "offline queue must not persist raw QR tokens"
)

# 0.4.2 hot-path and launcher identity contract
vault = (ROOT / "src-tauri/src/vault.rs").read_text(encoding="utf-8")
prepare_android = (ROOT / "scripts/prepare-android.py").read_text(encoding="utf-8")
assert 'show_mode_store: RwLock<Option<ShowModeStore>>' in native
assert 'operator_vault_password: RwLock<Option<Zeroizing<Vec<u8>>>>' in native
assert 'SHOW_MODE_SYNC_CONCURRENCY: usize = 4' in native
assert native.count('binary_search_by') >= 2
assert '.buffer_unordered(SHOW_MODE_SYNC_CONCURRENCY)' in native
assert 'save_show_mode_bytes_with_password' in native and 'load_show_mode_with_password' in native
assert 'pub fn operator_password(' in vault
assert 'save_show_mode_bytes_with_password' in vault
assert 'load_show_mode_with_password' in vault
for icon in ['icon.png', 'icon.ico', 'icon.icns', 'virya-signal.svg']:
    assert (ROOT / 'src-tauri/icons' / icon).is_file(), f'missing professional icon asset: {icon}'
for icon in ['ic_launcher_foreground.png', 'play-store-512.png']:
    assert (ROOT / 'src-tauri/launcher-assets/android' / icon).is_file(), (
        f'missing professional Android icon asset: {icon}'
    )
assert 'mipmap-anydpi-v33' in prepare_android and '<monochrome' in prepare_android
assert 'icons/icon.ico' in config and 'icons/icon.icns' in config

print("principal contract: PASS")
