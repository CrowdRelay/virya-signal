# Virya Signal

Tauri 2 mobile client for Virya Signal and CrowdRelay live operations. The UI is
written in Leptos/WASM; credentials, network access, wallet state and operator
actions live in the native Rust shell.

## 0.4.2 scope

- fan-side VIRYA AREA wallet and game progress without exposing drop coordinates;
- owner-only CrowdRelay operations cockpit with audited retries;
- partial-source degradation for operations data instead of an all-or-nothing screen;
- Stronghold-backed stable AREA identity and 37-command IPC contract guards.

## Toolchain

- Rust 1.97.0 (pinned in `rust-toolchain.toml`)
- Tauri CLI 2.11.4
- Trunk 0.21.14
- Java 21, Android API 36 and NDK 27.2.12479018

With rustup installed, the repository selects the pinned Rust toolchain
automatically.

## Local checks

```bash
cargo fmt --all --check
python3 scripts/static-check.py
python3 scripts/test-principal-contract.py
python3 -m unittest discover -s scripts -p 'test_*.py'
node scripts/test-boot.mjs
cargo check --locked -p crowdrelay-mobile-ui --target wasm32-unknown-unknown
cargo clippy --locked -p crowdrelay-mobile-ui --target wasm32-unknown-unknown -- -D warnings
trunk build --release
python3 scripts/check-web-dist.py dist
cargo check --locked -p crowdrelay-mobile
cargo test --locked -p crowdrelay-mobile
cargo clippy --locked -p crowdrelay-mobile --all-targets -- -D warnings
```

To apply reviewed compiler fixes and formatting in one pass, run
`bash scripts/quality-fix.sh`.

The native Linux checks also require the packages installed by
`.github/workflows/check.yml` (WebKitGTK, AppIndicator, librsvg and patchelf).

## Android development

```bash
cargo tauri android init --ci --skip-targets-install
cargo tauri android dev
```

Every push to `main` runs `Android Smoke Build`. It builds only the modern
ARM64 target, strips debug symbols and enforces a 150 MiB ceiling to keep
feedback fast and prevent oversized APK regressions. CI also rejects accidental
extra ABIs, invalid APKs and native libraries without 16 KiB page alignment. It uploads:

- GitHub artifact: `virya-signal-android-debug`
- APK: `virya-signal-debug.apk`
- checksum: `virya-signal-debug.apk.sha256`

The smoke APK is a debug build for direct testing. Direct signed APKs and the
initial Google Play bundle intentionally use ARM64 only. That covers modern
Android phones while avoiding three redundant native compilations and keeping
the bundle small. Add extra ABIs only when device analytics justify their cost.

## Signed APK

Add `ANDROID_KEYSTORE_BASE64`, `ANDROID_KEY_ALIAS` and
`ANDROID_KEY_PASSWORD` as GitHub Actions secrets. Then run the
`Android Signed APK` workflow, or push a tag such as:

```bash
git tag apk-v0.4.2
git push origin apk-v0.4.2
```

The direct-download release is ARM64-only, R8/resource-shrunk and capped at 100
MiB. The release file is named `virya-signal-0.4.2.apk`. Never replace or commit the
upload keystore: Android updates must keep the same application identifier and
signing certificate.

## Release behavior

`scripts/set-release-version.py` updates the Tauri bundle version without
rewriting Cargo manifests or invalidating the dependency cache. Android version
codes are derived monotonically from semantic versions. Build collection rejects
missing or ambiguous outputs instead of silently uploading the first file found.

The installed application name is **Virya Signal**. Its existing application ID
remains `music.virya.control` so upgrades stay compatible.

## Security model

- API and wallet secrets are stored through Stronghold, not in the webview.
- ticket QR tokens remain in a zeroizing native cache; the webview receives only
  a boolean availability flag and an SVG generated after an explicit tap;
- expensive vault/password work runs outside the asynchronous runtime workers;
- operator and fan sections fetch independently and render their own skeletons;
- the launcher renders immediately while vault status checks finish in the background;
- splash readiness uses an early listener, persistent DOM state and mounted-launcher
  observation, so Android WebView cannot lose a one-shot ready event;
- native bridge calls wait for Tauri injection and have bounded deadlines instead of
  leaving component loaders stuck forever;
- public concert/city reads use bounded persistent caches, HTTP validators and
  HTTP/2 compression; cached data can render before a network round-trip;
- overlapping read refreshes are latest-wins and cannot overwrite newer UI state;
- dynamic event, campaign and wallet lists use keyed DOM reconciliation;
- wallet failures are isolated, bounded to eight seconds each and loaded eight at
  a time, so one stale order cannot hide valid tickets or outlive the IPC deadline;
- QR rendering runs on the blocking pool rather than occupying async runtime workers;
- Rustls uses the compact Ring crypto provider instead of AWS-LC on mobile;
- mutation commands are serialized to prevent lost state updates;
- external URLs and API base URLs are validated before native use;
- response, token and user-input sizes are bounded;
- production API and external links require HTTPS;
- closing or explicitly locking a profile clears sensitive in-memory UI state.

Run the weekly `Security audit` workflow and review Dependabot pull requests
before releasing.

## Device profiling

Connect one unlocked Android device with USB debugging enabled, then run:

```bash
bash scripts/profile-android.sh music.virya.control artifacts/pixel8-profile 5
```

The script performs a warm-up plus repeated cold starts and saves aggregated startup
time, CPU, memory, frame statistics, package metadata and `[virya:boot]` /
`[virya:ipc]` diagnostics under `artifacts/`.
See `QUALITY.md` for budgets, architecture decisions and the release checklist.
