#!/usr/bin/env bash
set -euo pipefail

cargo clippy --fix --locked -p crowdrelay-mobile-ui --target wasm32-unknown-unknown --allow-dirty --allow-staged -- -D warnings
cargo clippy --fix --locked -p crowdrelay-mobile --all-targets --allow-dirty --allow-staged -- -D warnings
cargo fmt --all
cargo fmt --all --check
cargo clippy --locked -p crowdrelay-mobile-ui --target wasm32-unknown-unknown -- -D warnings
cargo test --locked -p crowdrelay-mobile
cargo clippy --locked -p crowdrelay-mobile --all-targets -- -D warnings
