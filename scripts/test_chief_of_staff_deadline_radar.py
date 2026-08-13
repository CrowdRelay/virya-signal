#!/usr/bin/env python3
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
contract = (ROOT / "crates/virya-signal-contracts/src/autopilot.rs").read_text()
ui = (ROOT / "src/app/operator.rs").read_text()
labels = (ROOT / "src/app/operator/autopilot_labels.rs").read_text()
pl = (ROOT / "src/i18n/pl.rs").read_text()
en = (ROOT / "src/i18n/en.rs").read_text()

assert "ChiefOfStaffAttentionItem" in contract and "attention_items" in contract
assert "autopilot_deadline_radar" in ui and "attention_items.into_iter().take(6)" in ui
assert "autopilot_attention_label" in labels and "autopilot_urgency_label" in labels
for key in ("autopilot_deadline_radar", "autopilot_urgency_overdue", "autopilot_attention_funding"):
    assert key in pl and key in en
print("CHIEF_OF_STAFF_DEADLINE_RADAR=PASS")
