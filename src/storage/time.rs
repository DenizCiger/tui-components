use std::time::{SystemTime, UNIX_EPOCH};

pub fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or(0)
}

pub fn is_expired(timestamp: u64, ttl_ms: u64) -> bool {
    now_ms().saturating_sub(timestamp) > ttl_ms
}
