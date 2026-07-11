use axum::routing::get;
use axum::{Json, Router};
use serde_json::json;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::SystemTime;

use crate::AppState;

static START_TIME: AtomicU64 = AtomicU64::new(0);

pub fn router() -> Router<Arc<AppState>> {
    START_TIME.compare_exchange(0, epoch_secs(), Ordering::Relaxed, Ordering::Relaxed).ok();
    Router::new().route("/", get(health))
}

fn epoch_secs() -> u64 {
    SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default().as_secs()
}

fn read_memory_kb() -> String {
    let status = std::fs::read_to_string("/proc/self/status").unwrap_or_default();
    for line in status.lines() {
        if let Some(val) = line.strip_prefix("VmRSS:") {
            return val.trim().to_string();
        }
    }
    "unknown".into()
}

async fn health() -> Json<serde_json::Value> {
    let started = START_TIME.load(Ordering::Relaxed);
    let uptime = if started > 0 { epoch_secs().saturating_sub(started) } else { 0 };

    Json(json!({
        "status": "ok",
        "memory": read_memory_kb(),
        "uptime_seconds": uptime,
    }))
}
