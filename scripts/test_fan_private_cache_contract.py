#!/usr/bin/env python3
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
vault=(ROOT/'src-tauri/src/vault.rs').read_text()
commands=(ROOT/'src-tauri/src/commands/fan/session_commerce.rs').read_text()
api=(ROOT/'src-tauri/src/api/fan.rs').read_text()
support=(ROOT/'src/app/support.rs').read_text()
generated=(ROOT/'crates/virya-signal-contracts/src/fan_wire.generated.rs').read_text()
checks={
'encrypted': 'FAN_HOME_CACHE_KEY' in vault and 'save_fan_home_cache_with_password' in vault,
'nonblocking': 'encrypted home snapshot ignored' in commands and 'Result<Option<FanHomeData>' in commands,
'revalidate': 'effective_age = age.max(FAN_HOME_CACHE_TTL)' in api,
'cached_first': 'fan_cached_home' in support and support.index('fan_cached_home') < support.index('"fan_home"'),
'contract_pin': '@crowdrelay-openapi-sha256' in generated,
}
for name, ok in checks.items(): print(f'FAN_PRIVATE_CACHE_{name.upper()}={"PASS" if ok else "FAIL"}')
if not all(checks.values()): raise SystemExit(1)
print(f'FAN_PRIVATE_CACHE_CONTRACT=PASS checks={len(checks)}')

cache=(ROOT/'src-tauri/src/api/cache.rs').read_text()
fan_api=(ROOT/'src-tauri/src/api/fan.rs').read_text()
assert 'MAX_FUTURE_CLOCK_SKEW' in cache
assert 'future_skew > 5 * 60' in fan_api
print('FAN_CACHE_CLOCK_SKEW_GUARD=PASS')
