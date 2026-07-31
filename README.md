# Virya Control Mobile

Private iOS/Android application for the Virya team, backed by CrowdRelay.

## Delivered MVP

- encrypted operator setup and local PIN unlock;
- separate `owner` and `staff` device profiles;
- upcoming-event dashboard and concert QR counters;
- native mobile QR scanner for admission passes;
- manual admission fallback;
- ticket-sale overview, inventory, revenue and recent orders;
- merch coupon redemption at the stand;
- owner-only pass issue and revoke actions;
- fan-area shell with public events, ready for fan auth/wallet in phase 2.

The UI is Leptos/WASM. HTTP, validation, authorization and secret handling live in the Rust Tauri layer. The bearer token is encrypted in an IOTA Stronghold vault and is not returned to the UI after unlock.

## Development prerequisites

- stable Rust compatible with `rust-version` in Cargo manifests;
- `rustup target add wasm32-unknown-unknown`;
- `cargo install trunk --locked`;
- `cargo install tauri-cli --version '^2' --locked`;
- Xcode for iOS and Android Studio/JDK/SDK/NDK for Android.

## Start

```bash
cargo tauri android init
cargo tauri android dev

# or on macOS
cargo tauri ios init
cargo tauri ios dev
```

Desktop UI preview:

```bash
cargo tauri dev
```

Camera scanning is intentionally available only on iOS/Android. Use manual references in desktop preview.

Every push is checked by `.github/workflows/check.yml`: Rust formatting, WASM UI compilation, native Tauri compilation/tests, Clippy and static IPC contracts.

## Pairing the first devices

1. Deploy the supplied CrowdRelay mobile patch.
2. Owner device: use `CROWDRELAY_ADMIN_API_KEY` and role `owner`.
3. Band-member devices: use `CROWDRELAY_STAFF_API_KEY` and role `staff`.
4. Set a unique local PIN on every phone.

Do not place either token in source code, CI variables used by the mobile build, screenshots, or public app configuration.

## Production hardening milestone

The MVP uses an encrypted static staff credential, which is acceptable for a small private team but rotates all devices together. Phase 1.1 should replace it with one-time pairing QR codes and short-lived per-device sessions backed by CrowdRelay `workspace_member_sessions`.

## Validation boundary of this delivery

The generation environment did not contain Rust, Xcode or the Android SDK and could not download a toolchain, so no local `cargo check` or device build is claimed. JSON/TOML configuration, required files, bracket integrity, IPC command registration and API-path contracts were checked locally. The included GitHub Actions workflow performs the real Rust compilation immediately after the repository is pushed.
