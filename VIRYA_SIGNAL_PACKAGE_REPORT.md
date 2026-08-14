# Virya Signal package report — 2026-08-14

Baseline: `virya-ecosystem-20260814-234133.zip` (Virya Signal 0.4.2).

## Included changes

- Android/desktop canonical full icon replaced with the approved, more distant and centered Signal Core artwork.
- Adaptive Android foreground scaled down to a safer centered footprint; the existing near-black adaptive background remains the launcher edge/background.
- All generated PNG/Android launcher variants regenerated from the new canonical artwork; ICO and ICNS refreshed too.
- Added an optimized local `bundle-stage-pack.webp` preview for the Stage Pack / `bundle-stage-pack` offer.
- Stage Pack cards deliberately prefer the bundled local preview, so a missing/broken remote merch image no longer leaves an empty card.
- Trunk copies the Stage Pack preview into the application distribution.
- Static contracts now require the Stage Pack asset and its UI wiring; a dedicated asset test guards the canonical icon dimensions and bundled preview contract.
- Existing Android FCM pipeline is preserved. Signed release verification requires `firebaseConfigured == true`; Firebase client configuration remains an external GitHub Actions secret and is not embedded in this source archive.

## Validation completed in packaging environment

- `python3 scripts/static-check.py` — PASS
- `python3 -m unittest discover -s scripts -p 'test_*.py'` — PASS (74 tests)
- `python3 scripts/check-ci-policy.py` — PASS
- `python3 scripts/source-size-ratchet.py` — PASS
- `python3 scripts/check-future-incompat.py` — PASS
- `bash -n scripts/*.sh` — PASS
- `node scripts/test-boot.mjs` — PASS
- Android push control-plane contract — PASS
- Package secret hygiene scan — PASS

Rust 1.97.1 / Cargo / Trunk are not installed in the packaging sandbox, so `cargo fmt`, `cargo test`, `cargo clippy` and `trunk build --release` were not executed locally. The repository's GitHub Actions `Check` workflow remains the canonical compiler/build gate and runs those stages before release.
