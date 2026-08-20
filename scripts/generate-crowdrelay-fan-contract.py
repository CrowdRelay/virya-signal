#!/usr/bin/env python3
"""Generate the deliberately narrow Fan Home wire contract from canonical OpenAPI.

This does not generate the whole CrowdRelay client: transport/retry/cache remain
hand-written. It pins only the small cross-process DTO surface that must not drift.
"""

from __future__ import annotations

import hashlib
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
OPENAPI = ROOT.parent / "crowdrelay" / "openapi" / "openapi.yaml"
OUT = ROOT / "crates" / "virya-signal-contracts" / "src" / "fan_wire.generated.rs"

if not OPENAPI.is_file():
    raise SystemExit(
        "canonical CrowdRelay OpenAPI not found at "
        f"{OPENAPI}; run the generator from an ecosystem checkout where "
        "crowdrelay and virya-signal are siblings"
    )

raw = OPENAPI.read_bytes()
text = raw.decode("utf-8")
EXPECTED_REQUIRED = {
    "FanHomeSnapshot": {"schema_version", "generated_at", "profile", "next_event", "synesthesia", "referral", "counts", "recommended_action"},
    "FanHomeProfile": {"display_name", "locale", "primary_city"},
    "FanHomeEvent": {"slug", "title", "venue", "city", "starts_at", "doors_at", "ends_at", "phase", "ticket_url", "interested", "has_pass", "has_paid_ticket", "ticket_sale_active"},
    "FanHomeSynesthesia": {"started", "completed", "rooms_completed", "client_total_elapsed_ms", "best_elapsed_ms", "completed_runs", "leaderboard_published", "leaderboard_rank", "linked_at", "reward_entered"},
    "FanHomeReferral": {"qualified", "pending"},
    "FanHomeCounts": {"event_interests", "active_passes", "paid_orders", "area_claims"},
    "FanRecommendedActionDetail": {"kind", "priority", "target", "expires_at", "reason"},
    "FanPushPreferences": {"shows", "releases", "community", "merch", "quiet_hours_enabled", "quiet_start", "quiet_end", "quiet_timezone"},
    "FanPushPreferencesUpdate": {"shows", "releases", "community", "merch", "quiet_hours_enabled", "quiet_start", "quiet_end"},
}
EXPECTED_OPTIONAL = {"FanHomeSnapshot": {"recommended"}}
EXPECTED_ACTIONS = {"continue_synesthesia", "open_wallet", "open_live_event", "share_post_show_feedback", "get_ticket", "follow_next_event", "explore_signal"}
EXPECTED_INLINE_PROPERTIES = {
    "FanHomeEvent": {
        "slug": "type: string", "title": "type: string",
        "venue": "type: [string, 'null']", "city": "type: [string, 'null']",
        "starts_at": "type: string, format: date-time",
        "doors_at": "type: [string, 'null'], format: date-time",
        "ends_at": "type: [string, 'null'], format: date-time",
        "phase": "type: string, enum: [upcoming, live, afterglow]",
        "ticket_url": "type: [string, 'null']", "interested": "type: boolean",
        "has_pass": "type: boolean", "has_paid_ticket": "type: boolean",
        "ticket_sale_active": "type: boolean",
    },
    "FanHomeSynesthesia": {
        "started": "type: boolean", "completed": "type: boolean",
        "rooms_completed": "type: integer, minimum: 0, maximum: 11",
        "client_total_elapsed_ms": "type: [integer, 'null'], format: int64, minimum: 0",
        "best_elapsed_ms": "type: [integer, 'null'], format: int64, minimum: 0",
        "completed_runs": "type: integer, format: int64, minimum: 0",
        "leaderboard_published": "type: boolean",
        "leaderboard_rank": "type: [integer, 'null'], format: int64, minimum: 1",
        "linked_at": "type: [string, 'null'], format: date-time",
        "reward_entered": "type: boolean",
    },
    "FanHomeProfile": {field: "type: [string, 'null']" for field in ("display_name", "locale", "primary_city")},
    "FanHomeReferral": {field: "type: integer, format: int64, minimum: 0" for field in ("qualified", "pending")},
    "FanHomeCounts": {field: "type: integer, format: int64, minimum: 0" for field in ("event_interests", "active_passes", "paid_orders", "area_claims")},
    "FanRecommendedActionDetail": {
        "kind": "$ref: '#/components/schemas/FanRecommendedAction'",
        "priority": "type: integer, minimum: 0, maximum: 100",
        "target": "type: string, maxLength: 256",
        "expires_at": "type: [string, 'null'], format: date-time",
        "reason": "type: string, maxLength: 80",
    },
    "FanHomeSnapshot": {
        "schema_version": "type: integer, const: 1",
        "generated_at": "type: string, format: date-time",
        "profile": "$ref: '#/components/schemas/FanHomeProfile'",
        "synesthesia": "$ref: '#/components/schemas/FanHomeSynesthesia'",
        "referral": "$ref: '#/components/schemas/FanHomeReferral'",
        "counts": "$ref: '#/components/schemas/FanHomeCounts'",
        "recommended_action": "$ref: '#/components/schemas/FanRecommendedAction'",
    },
    "FanPushPreferences": {
        **{field: "type: boolean" for field in ("shows", "releases", "community", "merch", "quiet_hours_enabled")},
        "quiet_start": "type: string, pattern: '^[0-2][0-9]:[0-5][0-9]$'",
        "quiet_end": "type: string, pattern: '^[0-2][0-9]:[0-5][0-9]$'",
    },
    "FanPushPreferencesUpdate": {
        **{field: "type: boolean" for field in ("shows", "releases", "community", "merch", "quiet_hours_enabled")},
        "quiet_start": "type: string, pattern: '^[0-2][0-9]:[0-5][0-9]$'",
        "quiet_end": "type: string, pattern: '^[0-2][0-9]:[0-5][0-9]$'",
    },
}

def schema_block(name: str) -> str:
    marker = f"    {name}:"
    start = text.find(marker)
    if start < 0:
        raise SystemExit(f"missing canonical OpenAPI schema: {name}")
    tail = text[start + len(marker):]
    next_schema = re.search(r"^    [A-Za-z][A-Za-z0-9_]*:", tail, re.MULTILINE)
    return tail[: next_schema.start()] if next_schema else tail

def inline_list(block: str, key: str) -> set[str]:
    match = re.search(rf"^      {key}: \[(?P<items>[^]]*)\]", block, re.MULTILINE)
    if not match:
        raise SystemExit(f"canonical OpenAPI schema missing inline {key} list")
    return {item.strip().strip("'\"") for item in match.group("items").split(",") if item.strip()}

for schema, expected_required in EXPECTED_REQUIRED.items():
    block = schema_block(schema)
    if "\n      type: object" not in block or "\n      additionalProperties: false" not in block:
        raise SystemExit(f"{schema} must remain a closed object schema")
    actual_required = inline_list(block, "required")
    if actual_required != expected_required:
        raise SystemExit(f"{schema} required fields drifted: expected={sorted(expected_required)} actual={sorted(actual_required)}")
    for field in expected_required | EXPECTED_OPTIONAL.get(schema, set()):
        if f"        {field}:" not in block:
            raise SystemExit(f"{schema}.{field} missing from canonical OpenAPI properties")
    for field, expected in EXPECTED_INLINE_PROPERTIES[schema].items():
        actual = re.search(rf"^        {re.escape(field)}: \{{ (?P<body>[^}}]+) \}}$", block, re.MULTILINE)
        if actual is None or actual.group("body") != expected:
            found = actual.group("body") if actual else "multiline-or-missing"
            raise SystemExit(f"{schema}.{field} contract drifted: expected={expected!r} actual={found!r}")

snapshot = schema_block("FanHomeSnapshot")
for field, reference in (("next_event", "FanHomeEvent"), ("recommended", "FanRecommendedActionDetail")):
    marker = f"        {field}:\n          oneOf:\n            - {{ $ref: '#/components/schemas/{reference}' }}\n            - {{ type: 'null' }}"
    if marker not in snapshot:
        raise SystemExit(f"FanHomeSnapshot.{field} nullable reference contract drifted")

push = schema_block("FanPushPreferences")
timezone = "        quiet_timezone:\n          type: string\n          minLength: 1\n          maxLength: 64"
if timezone not in push:
    raise SystemExit("FanPushPreferences.quiet_timezone type/bounds drifted")

actions = inline_list(schema_block("FanRecommendedAction"), "enum")
if actions != EXPECTED_ACTIONS:
    raise SystemExit(f"FanRecommendedAction enum drifted: expected={sorted(EXPECTED_ACTIONS)} actual={sorted(actions)}")

digest = hashlib.sha256(raw).hexdigest()
body = f'''// @generated by scripts/generate-crowdrelay-fan-contract.py; do not hand-edit.\n// @crowdrelay-openapi-sha256 {digest}\nuse serde::{{Deserialize, Serialize}};\n\n#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]\n#[serde(rename_all = "snake_case")]\npub enum FanRecommendedAction {{\n    ContinueSynesthesia,\n    OpenWallet,\n    OpenLiveEvent,\n    SharePostShowFeedback,\n    GetTicket,\n    FollowNextEvent,\n    ExploreSignal,\n    #[default]\n    #[serde(other)]\n    Unknown,\n}}\n\n#[derive(Clone, Debug, Deserialize, Serialize)]\npub struct FanRecommendedActionDetail {{\n    pub kind: FanRecommendedAction,\n    pub priority: u8,\n    pub target: String,\n    pub expires_at: Option<String>,\n    pub reason: String,\n}}\n\n#[derive(Clone, Debug, Default, Deserialize, Serialize)]\npub struct FanHomeData {{\n    pub schema_version: u32,\n    pub generated_at: String,\n    pub profile: FanHomeProfile,\n    pub next_event: Option<FanHomeEvent>,\n    pub synesthesia: FanHomeSynesthesia,\n    pub referral: FanHomeReferral,\n    pub counts: FanHomeCounts,\n    pub recommended_action: FanRecommendedAction,\n    #[serde(default)]\n    pub recommended: Option<FanRecommendedActionDetail>,\n    #[serde(default)]\n    pub stale: bool,\n}}\n\nimpl FanHomeData {{\n    pub const SCHEMA_VERSION: u32 = 1;\n\n    pub fn has_supported_schema(&self) -> bool {{\n        self.schema_version == Self::SCHEMA_VERSION\n    }}\n}}\n\n#[derive(Clone, Debug, Default, Deserialize, Serialize)]\npub struct FanHomeProfile {{\n    pub display_name: Option<String>,\n    pub locale: Option<String>,\n    pub primary_city: Option<String>,\n}}\n\n#[derive(Clone, Debug, Deserialize, Serialize)]\npub struct FanHomeEvent {{\n    pub slug: String,\n    pub title: String,\n    pub venue: Option<String>,\n    pub city: Option<String>,\n    pub starts_at: String,\n    pub doors_at: Option<String>,\n    pub ends_at: Option<String>,\n    pub phase: String,\n    pub ticket_url: Option<String>,\n    pub interested: bool,\n    pub has_pass: bool,\n    pub has_paid_ticket: bool,\n    pub ticket_sale_active: bool,\n}}\n\n#[derive(Clone, Debug, Default, Deserialize, Serialize)]\npub struct FanHomeSynesthesia {{\n    pub started: bool,\n    pub completed: bool,\n    pub rooms_completed: i16,\n    pub client_total_elapsed_ms: Option<i64>,\n    pub best_elapsed_ms: Option<i64>,\n    pub completed_runs: i64,\n    pub leaderboard_published: bool,\n    pub leaderboard_rank: Option<i64>,\n    pub linked_at: Option<String>,\n    pub reward_entered: bool,\n}}\n\n#[derive(Clone, Debug, Default, Deserialize, Serialize)]\npub struct FanHomeReferral {{\n    pub qualified: i64,\n    pub pending: i64,\n}}\n\n#[derive(Clone, Debug, Default, Deserialize, Serialize)]\npub struct FanHomeCounts {{\n    pub event_interests: i64,\n    pub active_passes: i64,\n    pub paid_orders: i64,\n    pub area_claims: i64,\n}}\n'''

body += '\n#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]\n#[serde(rename_all = "camelCase")]\npub struct FanPushPreferences {\n    pub shows: bool,\n    pub releases: bool,\n    pub community: bool,\n    pub merch: bool,\n    pub quiet_hours_enabled: bool,\n    pub quiet_start: String,\n    pub quiet_end: String,\n    pub quiet_timezone: String,\n}\n\nimpl Default for FanPushPreferences {\n    fn default() -> Self {\n        Self {\n            shows: true,\n            releases: true,\n            community: true,\n            merch: true,\n            quiet_hours_enabled: false,\n            quiet_start: "22:00".to_owned(),\n            quiet_end: "08:00".to_owned(),\n            quiet_timezone: "Europe/Warsaw".to_owned(),\n        }\n    }\n}\n\n#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]\n#[serde(rename_all = "camelCase", deny_unknown_fields)]\npub struct FanPushPreferencesUpdate {\n    pub shows: bool,\n    pub releases: bool,\n    pub community: bool,\n    pub merch: bool,\n    pub quiet_hours_enabled: bool,\n    pub quiet_start: String,\n    pub quiet_end: String,\n}\n'

OUT.write_text(body, encoding="utf-8")
print(f"SIGNAL_FAN_CONTRACT=GENERATED openapi_sha256={digest}")
