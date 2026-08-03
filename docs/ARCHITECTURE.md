# Virya Signal architecture

Virya Signal is a Tauri 2 application with a Leptos/WASM presentation layer and a native Rust security boundary.

## Boundaries

- `src/`: reactive UI, bounded DTOs and typed command invocation;
- `src-tauri/src/api.rs`: CrowdRelay HTTP adapter;
- `src-tauri/src/vault.rs`: Stronghold-backed credential storage;
- `src-tauri/src/lib.rs`: native command composition, session state and mobile lifecycle;
- CrowdRelay: authoritative fan, ticket, admission, campaign and operations state.

The WebView never owns long-lived operator credentials or raw ticket QR capabilities. Native commands validate inputs and return the smallest response needed by the active screen.

## State ownership

Operator, fan and show-mode mutations are serialized independently. Public content may use bounded persistent caches. Private sessions are loaded from Stronghold and sensitive in-memory values use zeroizing containers where practical.

## Offline show mode

A prepared show session keeps a bounded local scan journal and synchronizes against CrowdRelay when connectivity returns. PostgreSQL remains authoritative for final redemption and conflict classification.

## Crash evidence path

```text
UI action
  -> persisted command name (arguments excluded)
  -> bounded native invocation
  -> WebView error/rejection overlay when catchable
  -> native panic file written atomically when Rust panics
  -> next-launch recovery reads report
  -> WebView persists and displays report
  -> native report acknowledged and deleted
```

A foreground heartbeat identifies an abnormal WebView/process termination even when no JavaScript or Rust stack can be captured.
