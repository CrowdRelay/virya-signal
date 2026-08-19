#!/usr/bin/env fish
set -l ROOT (git rev-parse --show-toplevel 2>/dev/null)
or begin
    echo "ERROR: not inside a git repository" >&2
    exit 1
end
cd "$ROOT"; or exit 1

python3 scripts/check-ci-policy.py; or exit $status
python3 scripts/source-size-ratchet.py; or exit $status
python3 scripts/static-check.py; or exit $status
python3 scripts/test_signal_v2_design.py; or exit $status
python3 scripts/test_fan_home_contract_parity.py; or exit $status
python3 scripts/test_ui_async_stability.py; or exit $status
node scripts/test-boot.mjs; or exit $status
cargo fmt --all --check; or exit $status
cargo check --locked -p virya-signal-ui --target wasm32-unknown-unknown; or exit $status

echo "SIGNAL_FAST_CHECK=PASS"
