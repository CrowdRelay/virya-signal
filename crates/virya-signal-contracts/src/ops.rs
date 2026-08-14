use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct QueueSummary {
    #[serde(default)]
    pub pending: i64,
    #[serde(default)]
    pub processing: i64,
    #[serde(default)]
    pub delivered_24h: i64,
    #[serde(default)]
    pub dead: i64,
    #[serde(default)]
    pub oldest_pending_seconds: i64,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct DatabaseRuntimeSummary {
    #[serde(default)]
    pub pool_size: u32,
    #[serde(default)]
    pub pool_idle: u32,
    #[serde(default)]
    pub pool_max: u32,
    #[serde(default)]
    pub server_version_num: i32,
    #[serde(default)]
    pub io_method: Option<String>,
    #[serde(default)]
    pub io_workers: Option<i32>,
    #[serde(default)]
    pub io_max_concurrency: Option<i32>,
    #[serde(default)]
    pub effective_io_concurrency: Option<i32>,
    #[serde(default)]
    pub maintenance_io_concurrency: Option<i32>,
    #[serde(default)]
    pub io_combine_limit_bytes: Option<i64>,
    #[serde(default)]
    pub io_max_combine_limit_bytes: Option<i64>,
    #[serde(default)]
    pub async_io_active: bool,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct AreaRuntimeSummary {
    #[serde(default)]
    pub credits_total: i64,
    #[serde(default)]
    pub vouchers_issued: i64,
    #[serde(default)]
    pub stale_voucher_reservations: i64,
    #[serde(default)]
    pub ticket_rewards_issued: i64,
    #[serde(default)]
    pub stale_ticket_reward_reservations: i64,
    #[serde(default)]
    pub legacy_imported_players: i64,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct HttpRequestSummary {
    #[serde(default)]
    pub requests: u64,
    #[serde(default)]
    pub errors_4xx: u64,
    #[serde(default)]
    pub errors_5xx: u64,
    #[serde(default)]
    pub average_ms: u64,
    #[serde(default)]
    pub p50_ms: u64,
    #[serde(default)]
    pub p95_ms: u64,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct OpsSummary {
    #[serde(default)]
    pub outbox: QueueSummary,
    #[serde(default)]
    pub deliveries: QueueSummary,
    #[serde(default)]
    pub http: HttpRequestSummary,
    #[serde(default)]
    pub database: DatabaseRuntimeSummary,
    #[serde(default)]
    pub area: AreaRuntimeSummary,
    #[serde(default)]
    pub schema_version: u32,
    #[serde(default)]
    pub release: String,
}
