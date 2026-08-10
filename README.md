# Virya Signal

Tauri 2 / Rust / Leptos mobile client for the VIRYA fan and staff ecosystem. The WebView owns presentation; the native shell owns credentials, bounded network access, encrypted local state and privileged operator actions.

## Product boundary

Fan surfaces: Signal identity/recovery, concerts, tickets, merch, AREA and Synesthesia entry. Staff surfaces: operational overview, ticket/admission scanning, campaigns, commerce, diagnostics, and the ViryaOS Autopilot exception cockpit / Chief of Staff. The app exposes approvals, cancellations, authority and measured effects, but business decisions remain in CrowdRelay domain code.

CrowdRelay remains authoritative for fan/event/ticket/draw/inventory state. The app does not duplicate Stripe, fulfillment, winner selection or consent logic. Synesthesia is launched as an external first-party album experience at `https://synesthesia.virya.music/?source=signal-app`.

## Security

- Stronghold stores profiles/credentials; secrets do not live in the WebView.
- fan and staff PINs accept 4–6 digits and protect only local profile unlock;
- staff pairing validates expiry and stores the imported profile natively;
- response/token/input sizes and external URL origins are bounded;
- ticket QR secrets are retained natively and rendered only on demand;
- read refreshes are cancellable/latest-wins; mutations are serialized;
- public event/city data uses bounded caches and degrades independently;
- anonymous feedback contains category, message and a random idempotency ID only.

## Toolchain

Pinned Rust is in `rust-toolchain.toml`. CI uses Tauri 2, Trunk, Java 21, Android API 36 and ARM64 Android output.

## Local gates

```bash
cargo fmt --all --check
python3 scripts/static-check.py
python3 -m unittest discover -s scripts -p 'test_*.py'
node scripts/test-boot.mjs
cargo check --locked -p virya-signal-ui --target wasm32-unknown-unknown
cargo clippy --locked -p virya-signal-ui --target wasm32-unknown-unknown -- -D warnings
trunk build --release
python3 scripts/check-web-dist.py dist
cargo check --locked -p virya-signal
cargo test --locked -p virya-signal
cargo clippy --locked -p virya-signal --all-targets -- -D warnings
```

Use `bash scripts/quality-fix.sh` only for reviewed formatting/compiler fixes.

## Android

```bash
cargo tauri android init --ci --skip-targets-install
cargo tauri android dev
```

CI builds ARM64 debug/signed artifacts and enforces package-size/alignment contracts. Build outputs belong in CI artifacts or local `/artifacts`, never in source control. The installed application ID remains `music.virya.control` for upgrade compatibility.

Translations are compile-time catalogs in `src/i18n/{pl,en}.rs`; regenerate the pre-WASM boot catalog with `python3 scripts/generate-boot-i18n.py` after changing boot-visible copy.
