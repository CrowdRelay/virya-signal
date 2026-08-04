#!/usr/bin/env bash
set -euo pipefail

cargo clippy --fix --locked -p virya-signal-ui --target wasm32-unknown-unknown --allow-dirty --allow-staged -- -D warnings
cargo clippy --fix --locked -p virya-signal --all-targets --allow-dirty --allow-staged -- -D warnings
cargo fmt --all
cargo fmt --all --check
cargo clippy --locked -p virya-signal-ui --target wasm32-unknown-unknown -- -D warnings
cargo test --locked -p virya-signal
cargo clippy --locked -p virya-signal --all-targets -- -D warnings
