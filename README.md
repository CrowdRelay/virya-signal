# Virya Signal

**Rust / Tauri 2 / Leptos mobile client** for the VIRYA fan and staff ecosystem.

Signal is the phone-side surface of the same system virya.music serves in a browser: a fan carries their identity, tickets, concerts, merch and album experiences in it, and staff carry ticket scanning, admission, campaigns and the ViryaOS Autopilot cockpit. The WebView owns presentation only — the native shell owns credentials, bounded network access, encrypted local state and privileged operator actions, and CrowdRelay remains authoritative for fan, event, ticket, draw and inventory state.

This repo is intentionally a client boundary, not a second backend. Business decisions remain in CrowdRelay domain/application code.

## Engineering snapshot

- **Deliberate trust boundary:** Stronghold-backed profiles and credentials stay native; secrets do not live in the WebView.
- **Bounded networking:** response/token/input sizes and external origins are constrained before privileged native operations proceed.
- **Concurrency discipline:** read refreshes are cancellable/latest-wins; mutations are serialized instead of racing optimistic client state.
- **Capability separation:** fan and staff flows use separate local profiles and backend authorities; ticket QR secrets are retained natively and rendered only when needed.
- **Offline/degraded behavior:** public event/city data uses bounded caches and can degrade independently from private state.
- **Contract-first ecosystem:** the app follows CrowdRelay's canonical API instead of re-implementing Stripe, consent, winner selection, fulfillment or inventory policy.
- **Release discipline:** CI validates native Rust, wasm32/Leptos, Trunk output, Android package identity, package size/alignment and production release plumbing.

## Features

Fan surfaces: Signal identity and recovery, concerts, tickets, merch, AREA and Synesthesia entry. Staff surfaces: operational overview, ticket/admission scanning, campaigns, commerce, diagnostics and the ViryaOS Autopilot exception cockpit / Chief of Staff.

Synesthesia launches as a separate first-party experience at `https://synesthesia.virya.music/?source=signal-app`.

### Security and state

- Stronghold stores profiles/credentials;
- fan and staff PINs accept 4–6 digits and protect only local profile unlock;
- staff pairing validates expiry before importing the profile natively;
- ticket QR secrets never become general WebView state;
- anonymous feedback carries only category, message and a random idempotency ID;
- public caches are bounded and private refreshes remain generation-aware.

## Tech stack

Rust (pinned in `rust-toolchain.toml`) with Tauri 2 as the native shell and Leptos compiled to wasm32 for the UI, built with Trunk. CI uses Java 21, Android API 36 and ARM64 Android output.

Translations are compile-time catalogs in `src/i18n/{pl,en}.rs`; regenerate the pre-WASM boot catalog with `python3 scripts/generate-boot-i18n.py` after changing boot-visible copy.

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

CI builds ARM64 debug/signed artifacts and enforces package-size/alignment contracts. Build outputs belong in CI artifacts or local `/artifacts`, never in source control.

The Android application ID is `music.virya.signal`, declared once as the `identifier` in `src-tauri/tauri.conf.json`. Scripts, workflows and Play upload derive it from that source; `scripts/test_android_application_id.py` fails if a copy drifts. Google Play treats the application ID as permanent identity after first upload.
