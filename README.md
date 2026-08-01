# Virya Signal

Tauri 2 mobile client for Virya Signal and CrowdRelay live operations. The UI is
written in Leptos/WASM; credentials, network access, wallet state and operator
actions live in the native Rust shell.

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
python3 -m unittest discover -s scripts -p 'test_*.py'
cargo check --locked -p crowdrelay-mobile-ui --target wasm32-unknown-unknown
cargo check --locked -p crowdrelay-mobile
cargo test --locked -p crowdrelay-mobile
cargo clippy --locked -p crowdrelay-mobile --all-targets -- -D warnings
```

The native Linux checks also require the packages installed by
`.github/workflows/check.yml` (WebKitGTK, AppIndicator, librsvg and patchelf).

## Android development

```bash
cargo tauri android init --ci --skip-targets-install
cargo tauri android dev
```

Every push to `main` runs `Android Smoke Build`. It builds only the modern
ARM64 target, strips debug symbols and enforces a 150 MiB ceiling to keep
feedback fast and prevent oversized APK regressions. It uploads:

- GitHub artifact: `virya-signal-android-debug`
- APK: `virya-signal-debug.apk`
- checksum: `virya-signal-debug.apk.sha256`

The smoke APK is a debug build for direct testing. Direct signed APKs also use
ARM64; Google Play bundles retain every supported Android ABI.

## Signed APK

Add `ANDROID_KEYSTORE_BASE64`, `ANDROID_KEY_ALIAS` and
`ANDROID_KEY_PASSWORD` as GitHub Actions secrets. Then run the
`Android Signed APK` workflow, or push a tag such as:

```bash
git tag apk-v0.2.1
git push origin apk-v0.2.1
```

The direct-download release is ARM64-only and capped at 150 MiB. The release
file is named `virya-signal-0.2.1.apk`. Never replace or commit the
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
- expensive vault/password work runs outside the asynchronous runtime workers;
- operator and fan sections fetch independently and render their own skeletons;
- Rustls uses the compact Ring crypto provider instead of AWS-LC on mobile;
- mutation commands are serialized to prevent lost state updates;
- external URLs and API base URLs are validated before native use;
- response, token and user-input sizes are bounded;
- production API and external links require HTTPS;
- closing or explicitly locking a profile clears sensitive in-memory UI state.

Run the weekly `Security audit` workflow and review Dependabot pull requests
before releasing.
