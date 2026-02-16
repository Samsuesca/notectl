use chrono::{DateTime, Local, TimeZone};

/// Convert a Unix timestamp (seconds since epoch) to a local DateTime.
///
/// Falls back to `Local::now()` if the timestamp is invalid or ambiguous.
pub fn timestamp_to_local(ts: i64) -> DateTime<Local> {
    Local.timestamp_opt(ts, 0).single().unwrap_or_else(Local::now)
}
