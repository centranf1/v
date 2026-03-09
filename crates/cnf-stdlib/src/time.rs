//! Time utilities for CENTRA-NF

use chrono::{Utc, TimeZone};

/// Timestamp (UTC, detik)
pub fn now_timestamp() -> i64 {
    Utc::now().timestamp()
}

/// Format ISO 8601
pub fn format_iso8601(ts: i64) -> String {
    let dt = Utc.timestamp_opt(ts, 0).single().unwrap_or_else(|| Utc.timestamp_opt(0, 0).single().unwrap());
    dt.to_rfc3339()
}

/// Stopwatch sederhana
pub struct Stopwatch {
    start: i64,
}

impl Stopwatch {
    pub fn start() -> Self {
        Self { start: now_timestamp() }
    }
    pub fn elapsed(&self) -> i64 {
        now_timestamp() - self.start
    }
}
