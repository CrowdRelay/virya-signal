#!/usr/bin/env python3
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
backend=(ROOT.parent/'crowdrelay/crates/crowdrelay-api/src/fan_context.rs').read_text()
ui=(ROOT/'src/app/fan_home.rs').read_text()
contract=(ROOT/'crates/virya-signal-contracts/src/fan.rs').read_text()
required=['live_admission_ready','ticket_sale_active','FanRecommendedActionDetail','recommended_action_detail']
assert all(x in backend for x in required)
assert 'synesthesia_incomplete' not in backend and 'action("continue_synesthesia"' not in backend
assert 'snapshot.recommended.as_ref()' in ui and 'FanTarget::parse' in ui
assert 'key == "event"' in contract and 'not_event' in contract
print('NEXT_BEST_ACTION_V2=PASS rolling=true typed_targets=true synesthesia=demoted')

fan=(ROOT/'crates/virya-signal-contracts/src/fan.rs').read_text()
assert 'fn event_slug(value: &str)' in fan
assert 'value.len() <= 128' in fan
assert "byte.is_ascii_lowercase()" in fan
print('TYPED_TARGET_SLUG_GUARD=PASS')
