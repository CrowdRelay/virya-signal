#!/usr/bin/env python3
from pathlib import Path
import json
import os
import re

try:
    if os.environ.get('VIRYA_FORCE_TOML_FALLBACK'):
        raise ModuleNotFoundError
    import tomllib  # type: ignore[no-redef]
except ModuleNotFoundError:
    tomllib = None

root = Path(__file__).resolve().parents[1]

# The repository can contain local build products after Tauri/Gradle/Cargo runs.
# Some generated caches use JSON Lines or concatenate multiple JSON documents and
# are not configuration owned by the application. Static validation must inspect
# source-controlled configuration, not arbitrary contents of target/gen/build.
_GENERATED_DIRS = {
    '.git', '.gradle', '.idea', '.venv', '__pycache__',
    'build', 'dist', 'node_modules', 'target', 'venv',
}
_GENERATED_PREFIXES = (Path('src-tauri/gen'),)


def source_files(pattern: str):
    for candidate in root.rglob(pattern):
        relative = candidate.relative_to(root)
        if any(part in _GENERATED_DIRS for part in relative.parts):
            continue
        if any(relative == prefix or prefix in relative.parents for prefix in _GENERATED_PREFIXES):
            continue
        yield candidate


for path in source_files('*.json'):
    try:
        json.loads(path.read_text())
    except json.JSONDecodeError as error:
        raise SystemExit(f'invalid source JSON {path.relative_to(root)}: {error}') from error
if tomllib is not None:
    for path in source_files('*.toml'):
        try:
            tomllib.loads(path.read_text())
        except Exception as error:
            raise SystemExit(f'invalid source TOML {path.relative_to(root)}: {error}') from error

required = [
    'src-tauri/src/lib.rs', 'src-tauri/src/api.rs', 'src-tauri/src/vault.rs',
    'src/app.rs', 'src/bridge.rs', 'src-tauri/capabilities/mobile.json',
    '.github/workflows/check.yml', '.github/workflows/mobile-smoke.yml',
    'rust-toolchain.toml', '.cargo/config.toml', 'scripts/collect-mobile-artifact.py', 'boot.js',
    'scripts/test-boot.mjs', 'scripts/check-web-dist.py',
    'scripts/configure-android-signing.py',
    'scripts/analyze-android-package.py',
    'scripts/profile-android.sh',
    'scripts/install-android-sdk.sh', 'scripts/quality-fix.sh',
    'src-tauri/icons/icon.png', 'src-tauri/icons/icon.ico', 'src-tauri/icons/icon.icns',
    'src-tauri/icons/virya-signal.svg',
    'src-tauri/launcher-assets/android/ic_launcher_foreground.png',
    'src-tauri/launcher-assets/android/play-store-512.png',
]
for item in required:
    if not (root / item).is_file():
        raise SystemExit(f'missing {item}')

ui = (root / 'src/app.rs').read_text()
native = (root / 'src-tauri/src/lib.rs').read_text()
api = (root / 'src-tauri/src/api.rs').read_text()
bridge = (root / 'src/bridge.rs').read_text()
invoked = set(re.findall(
    r'bridge::invoke(?:_timeout|_latest)?(?:::<.*?>)?\(\s*"([a-z_]+)"', ui, re.S
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
boot = (root / 'boot.js').read_text()
main = (root / 'src/main.rs').read_text()
if 'data-trunk rel="copy-dir" href="public"' in index:
    raise SystemExit('index.html references the optional public directory')
if 'class="boot-led"' not in index or '@keyframes boot-led' not in index:
    raise SystemExit('Virya Signal splash LED is missing or not animated')
if '<script src="boot.js" defer>' in index:
    raise SystemExit('boot listener must execute before the deferred WASM module')
boot_tag = '<script src="boot.js?v=0.4.2"></script>'
if boot_tag not in index or index.find(boot_tag) > index.find('data-trunk rel="rust"'):
    raise SystemExit('boot listener must be declared before the WASM entrypoint')
for contract in ['window.__VIRYA_BOOT__', 'data-virya-ready', '.app-shell .launcher', 'MutationObserver']:
    if contract not in boot:
        raise SystemExit(f'boot recovery contract is missing: {contract}')
if 'URUCHOM APLIKACJĘ PONOWNIE' in boot or '15_000' in boot:
    raise SystemExit('boot watchdog must not turn a slow but healthy mount into a fatal error')
if main.find('mount_to_body') > main.rfind('virya_app_mounted();'):
    raise SystemExit('WASM must publish readiness only after mounting the app')
if 'console_error_panic_hook::set_once();' not in main or '#[cfg(debug_assertions)]' in main:
    raise SystemExit('release startup panics must remain diagnosable')
if 'futures::join!' in ui or 'fn Splash()' in ui:
    raise SystemExit('startup must render immediately instead of waiting behind a second splash')
if 'serde_json::Value' in ui or 'serde_json::Value' in bridge:
    raise SystemExit('discarded IPC responses must not pull JSON values into the WASM UI')
if 'invoke_timeout::<SessionStatus' not in ui or 'invoke_timeout::<FanSessionStatus' not in ui:
    raise SystemExit('startup vault reads must have independent IPC deadlines')
if 'while (!(core = window.__TAURI__?.core)' not in bridge or 'Promise.race' not in bridge:
    raise SystemExit('Android IPC bridge wait/timeout contract is missing')
if 'node scripts/test-boot.mjs' not in (root / '.github/workflows/check.yml').read_text():
    raise SystemExit('CI must execute the boot race/recovery runtime test')
check_workflow = (root / '.github/workflows/check.yml').read_text()
if 'trunk build --release' not in check_workflow or 'scripts/check-web-dist.py dist' not in check_workflow:
    raise SystemExit('CI must enforce the optimized frontend size budget')
for wasm_feature in ['--enable-bulk-memory', '--enable-bulk-memory-opt', '--enable-nontrapping-float-to-int']:
    if wasm_feature not in index:
        raise SystemExit(f'Rust 1.97 wasm-opt compatibility flag is missing: {wasm_feature}')

toolchain_text = (root / 'rust-toolchain.toml').read_text()
if tomllib is not None:
    toolchain_ok = tomllib.loads(toolchain_text).get('toolchain', {}).get('channel') == '1.97.0'
else:
    toolchain_ok = re.search(r'^channel\s*=\s*["\']1\.97\.0["\']\s*$', toolchain_text, re.M) is not None
if not toolchain_ok:
    raise SystemExit('rust-toolchain.toml must pin Rust 1.97.0')
android_config_text = (root / '.cargo/config.toml').read_text()
if tomllib is not None:
    android_rustflags = tomllib.loads(android_config_text).get('target', {}).get('aarch64-linux-android', {}).get('rustflags', [])
    page_size_ok = any('max-page-size=16384' in flag for flag in android_rustflags)
else:
    page_size_ok = 'max-page-size=16384' in android_config_text
if not page_size_ok:
    raise SystemExit('Android Rust libraries must be linked for 16 KiB pages')

smoke = (root / '.github/workflows/mobile-smoke.yml').read_text()
for trigger_path in ['index.html', 'boot.js', 'Trunk.toml', 'styles.css', 'rust-toolchain.toml']:
    if f'- "{trigger_path}"' not in smoke:
        raise SystemExit(f'Android smoke workflow does not watch {trigger_path}')
if '--target aarch64' not in smoke or '--max-size-mib' not in smoke:
    raise SystemExit('Android smoke must build a bounded ARM64 APK')
if 'virya-signal-debug.apk' not in smoke:
    raise SystemExit('Android smoke artifact must be named Virya Signal')

signed_apk = (root / '.github/workflows/android-release-apk.yml').read_text()
if '--target aarch64' not in signed_apk or '--max-size-mib 100' not in signed_apk:
    raise SystemExit('Signed Android APK must be a bounded ARM64 build')

workflows = '\n'.join(path.read_text() for path in (root / '.github/workflows').glob('*.yml'))
if 'cargo install trunk' in workflows or 'cargo install tauri-cli' in workflows:
    raise SystemExit('workflow compiles a Tauri tool instead of installing a cached binary')
if 'toolchain: 1.88.0' in workflows or "tauri-cli --version '^2'" in workflows:
    raise SystemExit('workflow contains an obsolete or floating tool version')

native_manifest_text = (root / 'src-tauri/Cargo.toml').read_text()
ui_manifest_text = (root / 'Cargo.toml').read_text()
if tomllib is not None:
    native_manifest = tomllib.loads(native_manifest_text)
    ui_manifest = tomllib.loads(ui_manifest_text)
else:
    required_native_fragments = [
        'rust-version = "1.97"', 'rand = "0.10"', 'iota_stronghold = "2.1.0"',
        'features = ["v4"]', 'features = ["gzip", "http2", "json", "rustls-no-provider"]',
        'futures-util = { version = "0.3", default-features = false, features = ["alloc"] }',
    ]
    required_ui_fragments = [
        'rust-version = "1.97"', '[profile.dev]', 'panic = "abort"', 'strip = "symbols"',
    ]
    if not all(fragment in native_manifest_text for fragment in required_native_fragments):
        raise SystemExit('native Cargo manifest is missing a required pinned contract')
    if not all(fragment in ui_manifest_text for fragment in required_ui_fragments):
        raise SystemExit('UI Cargo manifest is missing a required pinned contract')
    native_manifest = {
        'package': {'rust-version': '1.97'},
        'dependencies': {
            'rand': '0.10', 'iota_stronghold': '2.1.0',
            'uuid': {'features': ['v4']},
            'reqwest': {'features': ['gzip', 'http2', 'json', 'rustls-no-provider']},
            'futures-util': {'default-features': False, 'features': ['alloc']},
        },
    }
    ui_manifest = {
        'package': {'rust-version': '1.97'},
        'profile': {'dev': {'strip': 'symbols', 'panic': 'abort'}},
        'dependencies': {},
    }
if native_manifest.get('package', {}).get('rust-version') != '1.97' or ui_manifest.get('package', {}).get('rust-version') != '1.97':
    raise SystemExit('Cargo manifests must require Rust 1.97')
if any(key.startswith('profile') for key in native_manifest):
    raise SystemExit('Cargo profiles must be declared at the workspace root')
dev_profile = ui_manifest.get('profile', {}).get('dev', {})
if dev_profile.get('strip') != 'symbols' or dev_profile.get('panic') != 'abort':
    raise SystemExit('mobile debug builds must omit native symbols and unwinding')
if native_manifest.get('dependencies', {}).get('rand') != '0.10':
    raise SystemExit('native shell must use the validated rand 0.10 API')
ui_dependencies = ui_manifest.get('dependencies', {})
if 'futures' in ui_dependencies or 'serde_json' in ui_dependencies:
    raise SystemExit('unused direct WASM dependencies futures/serde_json must stay removed')
ui_models = (root / 'src/models.rs').read_text()
if len(re.findall(r'#\[derive\([^\]]*(?:Deserialize, Serialize|Serialize, Deserialize)', ui_models)) != 1:
    raise SystemExit('WASM models must derive only the serde direction used over IPC')
native_dependencies = native_manifest.get('dependencies', {})
if native_dependencies.get('uuid', {}).get('features') != ['v4']:
    raise SystemExit('native UUID support must not compile unused serde integration')
reqwest = native_dependencies.get('reqwest', {})
if not {'gzip', 'http2', 'json', 'rustls-no-provider'}.issubset(reqwest.get('features', [])):
    raise SystemExit('native HTTP client must keep compression, HTTP/2 and explicit Rustls')
if native_dependencies.get('iota_stronghold') != '2.1.0' or 'tauri-plugin-stronghold' in native_dependencies:
    raise SystemExit('native vault must depend on Stronghold directly without the unused Tauri plugin')
futures_util = native_dependencies.get('futures-util', {})
if futures_util.get('default-features') is not False or futures_util.get('features') != ['alloc']:
    raise SystemExit('native stream utilities must not enable the futures executor')
vault = (root / 'src-tauri/src/vault.rs').read_text()
if 'use rand::Rng;' not in vault or 'use rand::RngCore;' in vault:
    raise SystemExit('vault must import rand 0.10 trait rand::Rng')
if not re.search(r'buffered\(8\)\s*\.collect::<Vec<_>>\(\)', native):
    raise SystemExit('wallet loading must isolate individual backend failures')
if 'WALLET_REQUEST_TIMEOUT: Duration = Duration::from_secs(8)' not in api:
    raise SystemExit('wallet requests must fit inside the IPC deadline')
if 'invoke_latest::<WalletBatch' not in ui or '35_000' not in ui:
    raise SystemExit('wallet IPC deadline must cover bounded parallel loading')
wallet_model = re.search(r'pub struct WalletTicket \{(.*?)\n\}', (root / 'src/models.rs').read_text(), re.S)
if 'attach_wallet_qrs' in native or (wallet_model and 'qr_svg' in wallet_model.group(1)):
    raise SystemExit('wallet QR codes must stay lazy and out of the refresh hot path')
if 'render_wallet_qr' not in ui or 'render_wallet_qr' not in registered:
    raise SystemExit('lazy wallet QR command is not connected end to end')
if 'events_fetch: Arc<Mutex<()>>' not in api or 'cities_fetch: Arc<Mutex<()>>' not in api:
    raise SystemExit('public cache misses must be coalesced')
if 'EVENTS_STALE_TTL' not in api or 'CITIES_STALE_TTL' not in api:
    raise SystemExit('public reads must retain a bounded stale-on-error fallback')
for cache_contract in ['IF_NONE_MATCH', 'IF_MODIFIED_SINCE', 'DiskPublicCache', 'persist_public_cache']:
    if cache_contract not in api:
        raise SystemExit(f'persistent conditional public cache is missing: {cache_contract}')
if 'invoke_latest::<WalletBatch' not in ui or 'viryaInvokeLatest' not in bridge:
    raise SystemExit('refresh requests must suppress superseded responses')
if 'isMinifyEnabled = true' not in (root / 'scripts/prepare-android.py').read_text() or 'isShrinkResources = true' not in (root / 'scripts/prepare-android.py').read_text():
    raise SystemExit('release Android builds must enable R8 and resource shrinking')
if 'gradle/actions/setup-gradle@v4' in workflows or 'gradle/actions/setup-gradle@v6' not in workflows:
    raise SystemExit('Android workflows must use the current Gradle cache action')
if 'Result<TicketWalletApi, AppError>' not in api or 'Vec<serde_json::Value>' in (root / 'src-tauri/src/models.rs').read_text():
    raise SystemExit('wallet IPC must use a narrow typed payload')
if 'qr_token: Option<String>' in ui_models or 'wallet_qr_tokens:' not in native:
    raise SystemExit('wallet QR secrets must stay outside the WebView payload')
for typed_contract in [
    'Result<ConcertQrOverview, AppError>',
    'Result<TicketingOverview, AppError>',
    'Result<ReferralProgress, AppError>',
    'Result<Vec<FanEventInterest>, AppError>',
    'Result<Option<AdmissionPass>, AppError>',
]:
    if typed_contract not in api:
        raise SystemExit(f'heavy IPC response lost its typed DTO: {typed_contract}')
if 'pub city: Option<serde_json::Value>' in (root / 'src-tauri/src/models.rs').read_text():
    raise SystemExit('public event city must stay typed across the native/WebView boundary')

android_workflows = [
    root / '.github/workflows/mobile-smoke.yml',
    root / '.github/workflows/android-release-apk.yml',
    root / '.github/workflows/android-play.yml',
    root / '.github/workflows/mobile-release.yml',
]
for workflow in android_workflows:
    text = workflow.read_text()
    if 'shared-key: android-arm64' not in text or 'path: ~/.cache/trunk' not in text:
        raise SystemExit(f'{workflow.name} does not share Android/Trunk caches')
    if 'mozilla-actions/sccache-action@v0.0.11' not in text or 'cache-targets: false' not in text:
        raise SystemExit(f'{workflow.name} does not use the shared compiler cache')
    if '--signing' in text and 'scripts/configure-android-signing.py' not in text:
        raise SystemExit(f'{workflow.name} bypasses validated signing configuration')
    if 'scripts/analyze-android-package.py' not in text or '--require-abi arm64-v8a' not in text:
        raise SystemExit(f'{workflow.name} does not validate Android package contents')
    if '--require-page-size 16384' not in text:
        raise SystemExit(f'{workflow.name} does not verify 16 KiB native libraries')
if '--aab --target aarch64' not in (root / '.github/workflows/android-play.yml').read_text():
    raise SystemExit('Google Play AAB must use the bounded ARM64 target')

print(f'static configuration and IPC contract check: OK ({len(invoked)} used / {len(registered)} registered commands)')
