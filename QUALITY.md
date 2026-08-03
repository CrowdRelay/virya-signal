# Quality gates and release evidence

This file records enforced budgets and reproducible checks. It deliberately does not claim benchmark results that are not committed as artifacts.

## Enforced gates

```sh
python3 -m unittest discover -s scripts -p 'test_*.py'
cargo fmt --all -- --check
cargo check --locked --target wasm32-unknown-unknown
cargo clippy --locked --target wasm32-unknown-unknown -- -D warnings
cargo test --locked --manifest-path src-tauri/Cargo.toml
cargo clippy --locked --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
```

Android CI additionally verifies:

- ARM64 smoke build;
- APK integrity;
- 16 KiB native page alignment;
- debug APK ceiling of 150 MiB;
- signed direct-download APK ceiling of 100 MiB.

## Device evidence

Run:

```sh
bash scripts/profile-android.sh music.virya.control artifacts/device-profile 5
```

Commit or attach the generated startup, CPU, memory, frame and package metadata when making a performance claim. Record device model, Android version, build SHA and whether the run was cold or warm.

## Release checklist

1. all Python contracts, WASM and native gates are green;
2. crash-ledger contract is green;
3. install over the previous APK and confirm vault compatibility;
4. exercise fan onboarding, Signal, wallet, scanner and show mode on a real device;
5. force-stop during a native call and verify next-launch diagnostics;
6. verify the canonical launcher icon and version code;
7. retain the APK checksum with the release artifact.
