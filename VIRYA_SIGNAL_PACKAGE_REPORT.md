# Virya Signal package report — 2026-08-16 principal/staff hardening

Baseline: `virya-ecosystem-20260816-095444.zip`.

## Release blocker fixed

- Removed the accidentally restored `src-tauri/launcher-assets/` legacy tree. Android launcher sources remain canonical under `src-tauri/icons/android`.
- Removed the retired `boot-initializer.mjs`; Signal stays on the default Trunk module loader plus `boot.js` lifecycle/error handling.
- `.gitignore` and `scripts/static-check.py` now make both retired paths tombstones so a future source package fails immediately if either returns.
- Mobile smoke workflow watches the root boot/i18n/artifact inputs that can change startup behavior.

## Principal / staff security and lifecycle pass

- Fan and staff are separate backend audiences over one device-scoped FCM token. Audience disable/forget no longer deletes the provider token and therefore cannot silently break the other principal.
- Fan disable, fan forget, staff disable and staff forget require backend de-registration confirmation. A successful HTTP response with `registered=true` is not treated as disabled.
- Forgetting a fan or staff profile requires that profile to be unlocked so its bearer is still available for remote endpoint cleanup. `lock` remains available for immediate local privacy without destroying the cleanup capability.
- Staff push has an explicit durable opt-in/opt-out preference and a dedicated async mutation mutex. Bootstrap sync, enable, disable and forget cannot race each other into re-registering a disabled endpoint.
- Staff `forget_device` is fail-closed: the encrypted bearer/vault is retained when remote push cleanup cannot be proven, allowing a retry rather than creating an orphan endpoint.
- Pairing preserves the backend staff device-session expiry in the encrypted operator profile. Settings warn within 24 hours and after expiry, while local offline show-mode unlock remains available by design.
- Staff pairing remains Staff-only. Owner-only native API methods retain explicit `require_owner(profile)?` boundaries; staff QR/checklist/show operations stay in the staff namespace.

## CrowdRelay companion hardening

Signal's staff-principal lifecycle depends on CrowdRelay, so the ecosystem package also includes:

- automatic invalidation of staff push endpoints whose `staff_device_sessions` row is revoked or expired;
- active staff-session revalidation while claiming a push and again immediately before provider handoff;
- compatibility for static owner/staff API-key principals, which have no `staff_device_sessions` row;
- a source-size-safe split of Beacon Signal token/locale/radius helpers into `beacon_signal/helpers.rs` without changing its API contract.

## Validation completed in this packaging environment

- Signal static IPC/config: `72 active / 75 registered`, `3 compat-only`.
- Signal Python contract/unit suite: `105/105 PASS`.
- Signal boot runtime contract: PASS.
- Signal source-size ratchet: PASS.
- Signal CI policy: PASS.
- Signal shell syntax: PASS.
- CrowdRelay Python contract suite: `231 PASS`, `2 intentional skips`.
- CrowdRelay runtime umbrella: PASS.
- CrowdRelay OpenAPI/assets validation: `228 paths`, PASS.
- Virya web unit suite: `137/137 PASS`; static audits PASS.
- Synesthesia fast canonical validation: PASS.

The packaging runner has no `cargo`, `rustc` or `rustfmt`, and external Rust bootstrap is DNS-blocked. Therefore the real Rust compiler gates (`cargo fmt --check`, Clippy `-D warnings`, Rust tests/Tauri build) are not claimed here. The source-level release gates above are green and the package is intended to be followed by the repository's compiler-backed CI.
