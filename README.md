# Virya Signal

Tauri 2 mobile client for Virya Signal and CrowdRelay live operations. The UI is
written in Leptos/WASM; credentials, network access, wallet state and operator
actions live in the native Rust shell.

## 0.4.2 scope

- fan-side VIRYA AREA wallet and game progress without exposing drop coordinates;
- owner-only CrowdRelay operations cockpit with audited retries;
- partial-source degradation for operations data instead of an all-or-nothing screen;
- Stronghold-backed stable AREA identity and typed IPC contract guards;
- fan-first ticket checkout with multi-tier selection, Stripe handoff and automatic wallet persistence;
- first-class fan merch tab backed by authoritative CrowdRelay inventory and the existing Virya store;
- exact web-store bundles, a two-column mobile catalog and anonymous in-app feedback for fan and owner views;
- compile-time PL/EN catalogs with a persistent language switch shared by the WASM UI and native Tauri errors.

## Internationalization

Polish is the default language and English can be selected in both fan and staff
settings. The selection is stored under `virya:language:v1`; changing it reloads only
the interface, while encrypted profiles, sessions, wallets and cached data remain
untouched. The selected locale crosses the first launcher IPC, so native validation,
network errors, offline scanning and the WebView use the same language.

Translations have no runtime JSON parser or hash map. Both the Leptos/WASM crate and
the native Tauri crate compile the same static catalogs:

- `src/i18n/pl.rs`
- `src/i18n/en.rs`

Add the same key to those two files and reference it with `tr("key")`; use
`i18n::format`/native `i18n::replace` only for bounded dynamic placeholders. The
pre-WASM splash is generated from the same catalogs with:

```bash
python3 scripts/generate-boot-i18n.py
```

`static-check.py` and `test_i18n.py` reject missing keys, duplicate keys, mismatched
PL/EN placeholders, stale boot output and Polish runtime copy left outside the
catalogs.

## Fan commerce

The fan area exposes **Koncerty**, **Sklep** and **Bilety** as first-class tabs.
Every concert card opens a typed ticket flow. For Virya-owned sales the native
shell fetches the current offer from CrowdRelay, validates its bounds, creates
the existing Virya Stripe Checkout session and stores the order credential in
the encrypted fan profile before the browser handoff. The WebView receives only
the Stripe URL, public order ID/reference and expiry — the automatic checkout
response never contains the checkout token. The pre-existing manual wallet-recovery
form still accepts a token explicitly pasted by the user and passes it directly to
the native import command.
External organizer sales fall back to the event's ticket link or Virya event page.

Merch remains a thin mobile storefront over the authoritative CrowdRelay catalog.
The app renders two product cards per row and reads bundle names, contents, prices,
discounts and live availability from the same canonical definitions used by the web
store. Product and bundle cards open the existing Virya cart, so inventory, shipping,
Stripe, e-mail and accounting stay in one system rather than being duplicated in the
APK. Catalog and bundle reads remain lazy and run only after the Store tab is opened.

## Anonymous feedback

Fan and owner/operator settings expose the same feedback form. The WebView passes only
a bounded category and message to a native Tauri command. The native client creates a
random submission ID and posts to the fixed first-party endpoint
`https://virya.music/api/signal-feedback`; it never reads or attaches profile, session,
e-mail or operator identifiers. The endpoint validates the app origin, uses the existing
mail delivery ledger for idempotency and a keyed, durable network rate limiter (dedicated
`SIGNAL_FEEDBACK_RATE_SECRET` or the existing `AREA_AUTH_SECRET` fallback). It fails closed
when the rate limiter, mailer or lease store is unavailable. Standard hosting/network logs may still contain ordinary connection metadata,
so the UI does not promise network-level anonymity.

Operational prerequisites:

- `ticket_sales_enabled=true` and a configured Stripe secret on `virya.music`;
- the CrowdRelay event sale must be published/open with active ticket types;
- merch inventory must pass stocktake and be marked ready, otherwise the public
  catalog intentionally remains fail-closed with HTTP 503.

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
cargo check --locked -p virya-signal-ui --target wasm32-unknown-unknown
cargo clippy --locked -p virya-signal-ui --target wasm32-unknown-unknown -- -D warnings
trunk build --release
python3 scripts/check-web-dist.py dist
cargo check --locked -p virya-signal
cargo test --locked -p virya-signal
cargo clippy --locked -p virya-signal --all-targets -- -D warnings
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
- bundle responses are bounded, reject duplicate variants and accept only exact first-party store/image URLs;
- anonymous feedback carries only category, message and a random idempotency ID, never profile/session fields;
- ticket offers are response-bounded and checked for UUIDs, dates, counters,
  duplicate slugs and non-negative prices before the UI renders them;
- Stripe handoff accepts only exact `https://checkout.stripe.com` URLs and
  checkout tokens are validated and persisted natively before leaving the app;
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

## Engineering documentation

See [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) and [`docs/RELIABILITY.md`](docs/RELIABILITY.md).
