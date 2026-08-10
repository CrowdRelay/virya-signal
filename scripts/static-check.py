#!/usr/bin/env python3
from pathlib import Path
import json
import os
import re

from i18n_catalog import load_catalog_pair
from source_tree import read_app_source

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
    'src-tauri/src/lib.rs', 'src-tauri/src/api.rs', 'src-tauri/src/api/ticketing.rs', 'src-tauri/src/vault.rs',
    'src/app.rs', 'src/app/area.rs', 'src/bridge.rs', 'src-tauri/capabilities/mobile.json',
    '.github/workflows/check.yml', '.github/workflows/mobile-smoke.yml',
    'rust-toolchain.toml', '.cargo/config.toml', 'scripts/collect-mobile-artifact.py', 'boot.js', 'boot-i18n.js',
    'boot-initializer.mjs', 'scripts/generate-boot-i18n.py',
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

ui_main = read_app_source(root)
ui = ui_main
native = (root / 'src-tauri/src/lib.rs').read_text()
api = '\n'.join(
    path.read_text(encoding='utf-8')
    for path in sorted((root / 'src-tauri/src/api').rglob('*.rs'))
)
bridge = (root / 'src/bridge.rs').read_text()
boot = (root / 'boot.js').read_text()
index = (root / 'index.html').read_text()
pl_catalog, en_catalog = load_catalog_pair(root)
i18n_keys = set(pl_catalog)
invalid_i18n_keys = sorted(
    key for key in i18n_keys if not re.fullmatch(r'[a-z][a-z0-9_]*', key)
)
if invalid_i18n_keys:
    raise SystemExit(f'i18n identifiers must use English ASCII snake_case: {invalid_i18n_keys}')
i18n_source_paths = [
    *source_files('*.rs'),
    root / 'boot.js',
]
i18n_sources = '\n'.join(
    path.read_text(encoding='utf-8')
    for path in i18n_source_paths
    if 'src/i18n/pl.rs' not in path.as_posix()
    and 'src/i18n/en.rs' not in path.as_posix()
)
used_i18n_keys = set(re.findall(r'(?<![A-Za-z0-9_])tr\("([^"]+)"\)', i18n_sources))
used_i18n_keys.update(re.findall(r'i18n::(?:format|replace)\("([^"]+)"', i18n_sources))
used_i18n_keys.update(re.findall(r'text\("([^"]+)"', boot))
missing_i18n_keys = used_i18n_keys - i18n_keys
if missing_i18n_keys:
    raise SystemExit(f'code references missing i18n keys: {sorted(missing_i18n_keys)}')
placeholder = re.compile(r'\{([A-Za-z0-9_]+)\}')
for key in i18n_keys:
    if set(placeholder.findall(pl_catalog[key])) != set(placeholder.findall(en_catalog[key])):
        raise SystemExit(f'i18n placeholder mismatch for {key}')
for required_i18n_file in (
    'src/i18n.rs', 'src/i18n/pl.rs', 'src/i18n/en.rs',
    'src-tauri/src/i18n.rs', 'scripts/i18n_catalog.py',
):
    if not (root / required_i18n_file).is_file():
        raise SystemExit(f'missing {required_i18n_file}')
if 'virya:language:v1' not in (root / 'src/i18n.rs').read_text():
    raise SystemExit('language preference must be persisted locally')
if '<LanguageSwitch />' not in ui_main or ui_main.count('<LanguageSwitch />') < 2:
    raise SystemExit('language switch must be available in fan and staff settings')
if 'locale: i18n::current().code()' not in ui_main and 'locale: i18n::current().code().to_owned()' not in ui_main:
    raise SystemExit('fan API locale must follow the selected language')
if re.search(r'[ąćęłńóśźżĄĆĘŁŃÓŚŹŻ]', ui):
    raise SystemExit('Polish UI copy must live in the i18n catalog, not src/app.rs')
if re.search(r'[ąćęłńóśźżĄĆĘŁŃÓŚŹŻ]', bridge + boot + index):
    raise SystemExit('runtime WebView and boot copy must live in the i18n catalogs')
for native_path in sorted((root / 'src-tauri/src').rglob('*.rs')):
    runtime_native = native_path.read_text(encoding='utf-8').split('#[cfg(test)]', 1)[0]
    if re.search(r'[ąćęłńóśźżĄĆĘŁŃÓŚŹŻ]', runtime_native):
        raise SystemExit(
            f'Polish native runtime copy must live in i18n: {native_path.relative_to(root)}'
        )
native_i18n = (root / 'src-tauri/src/i18n.rs').read_text()
if '../../src/i18n/pl.rs' not in native_i18n or '../../src/i18n/en.rs' not in native_i18n:
    raise SystemExit('native and WASM must compile the same PL/EN catalogs')
if 'locale: i18n::current().code()' not in bridge or 'i18n::set_language(&locale)' not in (root / 'src-tauri/src/commands/misc.rs').read_text():
    raise SystemExit('selected locale must cross the first launcher IPC into native code')
invoked = set(re.findall(
    r'bridge::invoke(?:_timeout|_latest)?(?:::<.*?>)?\(\s*"([a-z_]+)"', ui, re.S
))
invoked.update(re.findall(
    r'bridge::invoke_unit\(\s*"([a-z_]+)"', ui, re.S
))
# Some native commands are intentionally hidden behind bridge helpers or the
# boot-time JS crash reporter. Count those literal calls too so the audit
# reflects the real IPC surface instead of under-reporting it.
invoked.update(re.findall(
    r'invoke(?:_timeout|_latest|_unit)?(?:::<[^>]+>)?\(\s*"([a-z_]+)"', bridge, re.S
))
invoked.update(re.findall(
    r"core\.invoke\(['\"]([a-z_]+)['\"]", bridge
))
registered_match = re.search(r'tauri::generate_handler!\[(.*?)\]', native, re.S)
if not registered_match:
    raise SystemExit('missing Tauri invoke handler')
registered = {x.strip().rsplit('::', 1)[-1] for x in registered_match.group(1).split(',') if x.strip()}
missing = invoked - registered
if missing:
    raise SystemExit(f'UI invokes unregistered commands: {sorted(missing)}')
compat_only_commands = {'session_status', 'public_events', 'public_cities'}
unreferenced = registered - invoked
if unreferenced != compat_only_commands:
    raise SystemExit(
        'Tauri IPC surface changed without an explicit compatibility decision: '
        f'unreferenced={sorted(unreferenced)} expected={sorted(compat_only_commands)}'
    )

required_paths = [
    'public/events?limit=50', 'public/merch/catalog', 'staff/admission/redeem', 'staff/coupons/redeem',
    'staff/event-qr/overview', 'admin/event-qr/overview',
    'admin/admission/passes', 'me/referral', 'me/events?limit=50',
    'public/ticket-orders/{order_id}/wallet', 'public/events/{event_slug}/tickets',
]

for path in required_paths:
    if path not in api:
        raise SystemExit(f'missing API contract path: {path}')

for contract in (
    'FanTicketCheckout',
    'FanTicketSale',
    'fan_ticket_sale',
    'fan_start_ticket_checkout',
    'continue_to_stripe_payment',
    'reopen_payment',
    '<FanNavButton tab=tab own=FanTab::Merch icon="shop" label=tr("store_tab")/>',
    'buy_in_store',
    'fan_merch_bundles',
    'submit_anonymous_feedback',
    'anonymous_feedback',
):
    if contract not in ui:
        raise SystemExit(f'fan commerce UX contract is missing: {contract}')

ticketing = (root / 'src-tauri/src/api/ticketing.rs').read_text()
native_fan = (root / 'src-tauri/src/commands/fan.rs').read_text()
frontend_models = (root / 'src/models.rs').read_text()
for contract in (
    'https://virya.music/api/ticket-checkout',
    '.header(ORIGIN, VIRYA_SITE_ORIGIN)',
    'checkout.stripe.com',
    'MAX_CHECKOUT_LINES',
    'MAX_CHECKOUT_QUANTITY',
    'public/events/{event_slug}/tickets',
):
    if contract not in ticketing:
        raise SystemExit(f'native ticket checkout hardening contract is missing: {contract}')
if 'checkout_token' in frontend_models:
    raise SystemExit('checkout token must never be deserialized into the WASM model')
if native_fan.find('persist_fan(&state, &profile).await?;') > native_fan.find('Ok(checkout)'):
    raise SystemExit('ticket wallet credential must be persisted before checkout is returned to the WebView')
if 'Zeroizing::new(response.checkout_token)' not in native_fan:
    raise SystemExit('ticket checkout token must be zeroized while crossing the native command')

for contract in (
    'fn new_operator_pin_is_valid',
    '(4..=6).contains(&pin.len())',
    'create_an_unlock_pin',
    'enter_4_6_digits_for_example_2580',
    'inputmode="numeric"',
    'vault::save_verified',
):
    if contract not in ui and contract != 'vault::save_verified':
        raise SystemExit(f'staff PIN regression contract is missing from UI: {contract}')
operator_command = (root / 'src-tauri/src/commands/operator.rs').read_text()
validation = (root / 'src-tauri/src/validation.rs').read_text()
vault = (root / 'src-tauri/src/vault.rs').read_text()
if 'vault::save_verified' not in operator_command:
    raise SystemExit('staff pairing must verify the persisted vault before reporting success')
if 'validate_new_operator_pin(&pin)?;' not in operator_command:
    raise SystemExit('new staff PINs must use the 4–6 digit native validator')
if 'operator_pin_survives_a_fresh_vault_round_trip' not in vault:
    raise SystemExit('staff PIN persistence regression test is missing')
if 'native_operator_pin_4_6' not in validation:
    raise SystemExit('native staff PIN validation contract is missing')

operator_nav = ui_main[ui_main.find('<nav class="overflow-menu">'):ui_main.find('<div class="content">')]
if 'OperatorTab::Signal' in operator_nav:
    raise SystemExit('staff Signal must live in the bottom navigation, not the overflow menu')
for contract in (
    '<nav class="bottom-nav four primary-four">',
    '<NavButton tab=tab own=OperatorTab::Signal icon="signal" label=tr("signal_tab") />',
):
    if contract not in ui:
        raise SystemExit(f'staff four-tab navigation contract is missing: {contract}')
css = (root / 'styles.css').read_text()
for contract in (
    '--control-radius: 10px',
    '.ghost { min-height: 48px;',
    '.advanced-config > .ghost { width: 100%; min-height: 54px;',
    'align-items: stretch',
    'height: 100%',
    '.ticket-pool-status {',
    '.area-native-map {',
):
    if contract not in css:
        raise SystemExit(f'mobile UX consistency contract is missing: {contract}')

for contract in (
    'tauri-plugin-geolocation = "2.3.2"',
    'tauri_plugin_geolocation::init()',
    'geolocation:allow-get-current-position',
    'geolocation:allow-request-permissions',
):
    source = (
        (root / 'src-tauri/Cargo.toml').read_text()
        + native
        + (root / 'src-tauri/capabilities/mobile.json').read_text()
    )
    if contract not in source:
        raise SystemExit(f'native AREA geolocation contract is missing: {contract}')
for contract in (
    'fan_area_challenge',
    'fan_area_claim',
    'collect_location_samples',
    'AreaGameScreen',
    'area_location_privacy',
):
    if contract not in ui + bridge + native_fan:
        raise SystemExit(f'native AREA game contract is missing: {contract}')

index = (root / 'index.html').read_text()
boot = (root / 'boot.js').read_text()
boot_initializer = (root / 'boot-initializer.mjs').read_text()
main = (root / 'src/main.rs').read_text()
if 'data-trunk rel="copy-dir" href="public"' in index:
    raise SystemExit('index.html references the optional public directory')
if 'class="boot-signal"' not in index or '@keyframes boot-pulse' not in index:
    raise SystemExit('Virya Signal splash LED is missing or not animated')
if '<script src="boot.js" defer>' in index:
    raise SystemExit('boot listener must execute before the deferred WASM module')
boot_i18n_tag = '<script src="boot-i18n.js?v=0.4.2-i18n-v1"></script>'
boot_tag = '<script src="boot.js?v=0.4.2-startup-v8"></script>'
if boot_i18n_tag not in index or index.find(boot_i18n_tag) > index.find(boot_tag):
    raise SystemExit('boot translations must load before the boot listener')
if boot_tag not in index or index.find(boot_tag) > index.find('data-trunk rel="rust"'):
    raise SystemExit('boot listener must be declared before the WASM entrypoint')
for contract in ['window.__VIRYA_BOOT__', 'data-virya-ready', '.app-shell .launcher', 'unhandledrejection', 'retry-blocked']:
    if contract not in boot:
        raise SystemExit(f'boot recovery contract is missing: {contract}')
if 'URUCHOM APLIKACJĘ PONOWNIE' in boot or '15_000' in boot:
    raise SystemExit('boot watchdog must not turn a slow but healthy mount into a fatal error')
if main.find('mount_to_body') > main.rfind('virya_app_mounted();'):
    raise SystemExit('WASM must publish readiness only after mounting the app')
if '#[wasm_bindgen(inline_js' in main or 'js_sys::Reflect' not in main:
    raise SystemExit('startup must not depend on a generated inline-js snippet module')
if 'data-initializer="boot-initializer.mjs"' not in index or 'boot()?.fail?.(error)' not in boot_initializer:
    raise SystemExit('Trunk WASM failures must be routed into the visible boot recovery UI')

# Bundled Trunk WASM is loaded through same-origin fetch().
tauri_config = json.loads((root / 'src-tauri/tauri.conf.json').read_text())
tauri_csp = tauri_config.get('app', {}).get('security', {}).get('csp', '')
connect_src = next(
    (
        directive.strip()
        for directive in tauri_csp.split(';')
        if directive.strip().startswith('connect-src ')
    ),
    '',
)
connect_tokens = connect_src.split()
if connect_tokens.count("'self'") != 1:
    raise SystemExit(
        "Tauri CSP connect-src must contain exactly one 'self' "
        "so Android WebView can fetch the bundled Trunk WASM"
    )
for required_connect_source in (
    'ipc:',
    'http://ipc.localhost',
    'https://signal-api.virya.music',
):
    if required_connect_source not in connect_tokens:
        raise SystemExit(
            f'Tauri CSP connect-src lost required source: '
            f'{required_connect_source}'
        )

if 'console_error_panic_hook::set_once();' not in main or '#[cfg(debug_assertions)]' in main:
    raise SystemExit('release startup panics must remain diagnosable')
if 'futures::join!' in ui or 'fn Splash()' in ui:
    raise SystemExit('startup must render immediately instead of waiting behind a second splash')

for contract in (
    'RwSignal::new(RootMode::Fan)',
    'RootMode::StaffGate',
    'verify_staff_access',
    'are_you_on_the_staff',
    '<NavGlyph icon=icon/>',
    '<StatusFailure',
    'status_failed=fan_status_failed',
):
    if contract not in ui:
        raise SystemExit(f'fan-first or staff-gate UX contract is missing: {contract}')
if ui.count('mode.set(RootMode::Team)') != 1 or 'Ok(()) => mode.set(RootMode::Team)' not in ui:
    raise SystemExit('operator UI must be reachable only after successful staff-gate verification')

native_client = (root / 'src-tauri/src/api/client.rs').read_text()
native_misc = (root / 'src-tauri/src/commands/misc.rs').read_text()
for contract in (
    'https://virya.music/api/staff/qr/login',
    '.header(ORIGIN, STAFF_GATE_ORIGIN)',
    '.redirect(reqwest::redirect::Policy::none())',
    'StatusCode::TOO_MANY_REQUESTS',
):
    if contract not in native_client:
        raise SystemExit(f'server-verified staff gate contract is missing: {contract}')
if 'Zeroizing::new(password)' not in native_misc:
    raise SystemExit('staff password must be zeroized immediately in the native command')

native_models = (root / 'src-tauri/src/models.rs').read_text()
for contract in (
    'fn normalize_compat_string',
    'deserialize_string_or_bytes',
    'referral_payload_accepts_legacy_byte_sequences_for_text_fields',
    'legacy_byte_sequences_are_migrated_to_strings',
    'accepts_node_buffer_objects',
):
    if contract not in native_models:
        raise SystemExit(f'StringOrBytes compatibility regression contract is missing: {contract}')
if 'serde_json::Value' in ui or 'serde_json::Value' in bridge:
    raise SystemExit('discarded IPC responses must not pull JSON values into the WASM UI')
launcher_startup = 'bridge::launcher_status' in ui
legacy_startup = (
    'invoke_timeout::<SessionStatus' in ui and 'invoke_timeout::<FanSessionStatus' in ui
)
if not launcher_startup and not legacy_startup:
    raise SystemExit('startup must read operator and fan vault status before rendering the launcher')
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
android_builder = (root / '.github/workflows/_android-build.yml').read_text()
for trigger_path in ['index.html', 'boot.js', 'boot-initializer.mjs', 'Trunk.toml', 'styles.css', 'rust-toolchain.toml']:
    if f'- "{trigger_path}"' not in smoke:
        raise SystemExit(f'Android smoke workflow does not watch {trigger_path}')
if "uses: ./.github/workflows/_android-build.yml" not in smoke or "build_kind: debug-apk" not in smoke:
    raise SystemExit('Android smoke must use the canonical reusable Android builder')
if '--target aarch64' not in android_builder or '--max-size-mib' not in android_builder:
    raise SystemExit('Reusable Android builder must create bounded ARM64 packages')
if 'package="artifacts/virya-signal-${VERSION}.${ext}"' not in android_builder:
    raise SystemExit('Reusable Android builder must produce canonical Virya Signal artifact names')

signed_apk = (root / '.github/workflows/android-release-apk.yml').read_text()
if "uses: ./.github/workflows/_android-build.yml" not in signed_apk or "build_kind: release-apk" not in signed_apk or "signed: true" not in signed_apk:
    raise SystemExit('Signed Android APK must use the canonical signed release builder')

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
        'rust-version = "1.97"', 'getrandom = "0.4"', 'iota_stronghold = "2.1.0"',
        'features = ["v4"]', 'features = ["gzip", "json", "rustls-no-provider"]',
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
            'getrandom': '0.4', 'iota_stronghold': '2.1.0',
            'uuid': {'features': ['v4']},
            'reqwest': {'features': ['gzip', 'json', 'rustls-no-provider']},
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
native_dependencies = native_manifest.get('dependencies', {})
if native_dependencies.get('getrandom') != '0.4' or 'rand' in native_dependencies:
    raise SystemExit('native shell must use the OS RNG directly without the rand facade')
ui_dependencies = ui_manifest.get('dependencies', {})
if 'futures' in ui_dependencies or 'serde_json' in ui_dependencies:
    raise SystemExit('unused direct WASM dependencies futures/serde_json must stay removed')
ui_models = (root / 'src/models.rs').read_text()
if len(re.findall(r'#\[derive\([^\]]*(?:Deserialize, Serialize|Serialize, Deserialize)', ui_models)) != 1:
    raise SystemExit('WASM models must derive only the serde direction used over IPC')
if native_dependencies.get('uuid', {}).get('features') != ['v4']:
    raise SystemExit('native UUID support must not compile unused serde integration')
reqwest = native_dependencies.get('reqwest', {})
if not {'gzip', 'json', 'rustls-no-provider'}.issubset(reqwest.get('features', [])):
    raise SystemExit('native HTTP client must keep compression, JSON and explicit Rustls')
if native_dependencies.get('iota_stronghold') != '2.1.0' or 'tauri-plugin-stronghold' in native_dependencies:
    raise SystemExit('native vault must depend on Stronghold directly without the unused Tauri plugin')
futures_util = native_dependencies.get('futures-util', {})
if futures_util.get('default-features') is not False or futures_util.get('features') != ['alloc']:
    raise SystemExit('native stream utilities must not enable the futures executor')
vault = (root / 'src-tauri/src/vault.rs').read_text()
if 'getrandom::fill(&mut salt)' not in vault or 'rand::' in vault:
    raise SystemExit('vault salt must come directly from the operating-system RNG')
native_src = '\\n'.join(
    path.read_text(encoding='utf-8')
    for path in sorted((root / 'src-tauri/src').rglob('*.rs'))
)
if 'const WALLET_FETCH_CONCURRENCY: usize = 8;' not in native_src or not re.search(
    r'buffered\(WALLET_FETCH_CONCURRENCY\)\s*\.collect::<Vec<_>>\(\)', native_src
):
    raise SystemExit('wallet loading must stay bounded to preserve the wallet IPC latency budget')
if 'WALLET_REQUEST_TIMEOUT: Duration = Duration::from_secs(12)' not in api:
    raise SystemExit('wallet requests must fit inside the IPC deadline')
if 'invoke_latest::<WalletBatch' not in ui or '35_000' not in ui:
    raise SystemExit('wallet IPC deadline must cover bounded parallel loading')
wallet_model = re.search(r'pub struct WalletTicket \{(.*?)\n\}', (root / 'src/models.rs').read_text(), re.S)
if 'attach_wallet_qrs' in native_src or (wallet_model and 'qr_svg' in wallet_model.group(1)):
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
if 'gradle/actions/setup-gradle@9c971963bec38e04b3d30dcc455b5382be2fdbfb' not in workflows:
    raise SystemExit('Android workflows must use the SHA-pinned Gradle v6.3.0 cache action')
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

native_models = (root / 'src-tauri/src/models.rs').read_text()
for source_name, source in [('native models', native_models), ('WASM models', ui_models)]:
    if '#[serde(default)]\n    pub errors_4xx: u64' not in source:
        raise SystemExit(f'{source_name} must decode CrowdRelay errors_4xx telemetry compatibly')
if 'summary.http.errors_4xx.to_string()' not in ui:
    raise SystemExit('staff Ops UI must surface CrowdRelay 4xx telemetry separately from 5xx')

for cache_contract in [
    'shared-key: android-arm64',
    'path: ~/.cache/trunk',
    'mozilla-actions/sccache-action@fc920bf0ec8de6ee65d409111f7ec508035751ba',
    'cache-targets: false',
    'scripts/configure-android-signing.py',
    'scripts/analyze-android-package.py',
    '--require-abi arm64-v8a',
    '--require-page-size 16384',
    '--aab --target aarch64',
    'artifact_manifest.py create artifacts android-artifact-manifest.json',
]:
    if cache_contract not in android_builder:
        raise SystemExit(f'reusable Android builder lost contract: {cache_contract}')
for wrapper_name in ['mobile-smoke.yml', 'android-release-apk.yml', 'android-play.yml', 'mobile-release.yml']:
    wrapper = (root / '.github/workflows' / wrapper_name).read_text()
    if 'uses: ./.github/workflows/_android-build.yml' not in wrapper:
        raise SystemExit(f'{wrapper_name} bypasses the canonical reusable Android builder')
if 'target:' not in (root / '.github/workflows/mobile-release.yml').read_text() or "github.event_name == 'workflow_dispatch'" not in (root / '.github/workflows/mobile-release.yml').read_text():
    raise SystemExit('mobile store release must gate iOS behind an explicit manual target')
for promotion_name in ['android-play.yml', 'android-release-apk.yml', 'mobile-release.yml']:
    promotion = (root / '.github/workflows' / promotion_name).read_text()
    if 'promoted/scripts/artifact_manifest.py verify' not in promotion:
        raise SystemExit(f'{promotion_name} must verify the exact downloaded artifact from its preserved scripts/ path')

print(f'static configuration and IPC contract check: OK ({len(invoked)} active / {len(registered)} registered commands; {len(unreferenced)} compat-only)')
