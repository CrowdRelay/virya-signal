# Architecture

```text
Leptos UI (WASM, no secrets)
        |
        | Tauri IPC commands
        v
Rust mobile shell
  - role checks
  - request validation
  - reqwest CrowdRelay client
  - in-memory unlocked session
  - encrypted Stronghold vault
        |
        | HTTPS + bearer + idempotency key
        v
CrowdRelay / signal-api.virya.music
  - PostgreSQL source of truth
  - atomic admission redemption
  - ticket inventory and Stripe reconciliation
  - coupons and audit events
```

## Trust boundaries

The frontend can ask the Rust shell to perform a named operation, but it cannot read the bearer token. The Rust shell constrains URL paths, identifiers and roles. CrowdRelay remains authoritative and repeats authorization server-side.

The fan section never reuses an operator token. Future fan login will use the existing CrowdRelay/Virya fan session and ticket-wallet contracts.
