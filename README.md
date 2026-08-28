# Virya Signal

**Rust / Tauri 2 / Leptos mobile client for the VIRYA fan and staff ecosystem.**

Signal is the phone-side surface of the same system virya.music serves in a browser. Fans carry identity, tickets, concerts, merch and album experiences; staff use ticket scanning, admission, campaigns, commerce and the ViryaOS operations cockpit. The WebView owns presentation, the native shell owns credentials and privileged actions, and CrowdRelay remains authoritative for business state.

## Features

- fan identity and recovery;
- concerts, tickets, merch, AREA and Synesthesia entry;
- staff operations overview and ticket/admission scanning;
- campaigns, commerce and diagnostics;
- ViryaOS exception cockpit / Chief of Staff;
- growth intelligence: the deterministic brain dispatches LLM workers
  (Reddit scanning, press pitches, social posts, community engagement) and
  the Signal app surfaces growth metrics and community engagement drafts;
- bounded offline caches and degraded public-data behavior;
- native-only credential and secret handling through Stronghold.

## Tech stack

Rust with Tauri 2 as the native shell and Leptos compiled to wasm32 for the UI, built with Trunk. Android release builds use Java 21, Android API 36 and ARM64 output. CrowdRelay is the canonical backend contract.

The Rust catalogs in `src/i18n/{pl,en}.rs` are canonical; the WASM UI loads them as runtime JSON catalogs at boot (`scripts/generate-boot-i18n.py`), so live language switching never re-downloads the binary.

## Security boundary

Secrets stay native and do not become general WebView state. Fan and staff profiles are separated, ticket QR secrets remain native, private refreshes are generation-aware, and external origins plus payload sizes are bounded before privileged operations proceed.

## License

See [`LICENSE`](LICENSE).

## Gates

CI (`.github/workflows/check.yml`) is the source of truth; locally run the target you touched:

```
cargo fmt --all --check
cargo check  --locked -p virya-signal-ui --target wasm32-unknown-unknown
cargo clippy --locked -p virya-signal-ui --target wasm32-unknown-unknown -- -D warnings
trunk build --release
cargo check  --locked -p virya-signal
cargo test   --locked -p virya-signal
cargo clippy --locked -p virya-signal --all-targets -- -D warnings
```

`scripts/fast-check.fish` runs the quick static subset.
