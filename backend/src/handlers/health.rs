<<<<<<< HEAD
use axum::extract::State;
use axum::routing::get;
use axum::{Json, Router};
use serde_json::json;
use sqlx::PgPool;
=======
use axum::routing::get;
use axum::{Json, Router};
use serde_json::json;
>>>>>>> origin/main
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

<<<<<<< HEAD
async fn db_stats(pool: &PgPool) -> serde_json::Value {
    json!({
        "boards": sqlx::query_scalar::<_, Option<i64>>("SELECT COUNT(*) FROM boards").fetch_one(pool).await.unwrap_or(None).unwrap_or(0),
        "cards": sqlx::query_scalar::<_, Option<i64>>("SELECT COUNT(*) FROM cards").fetch_one(pool).await.unwrap_or(None).unwrap_or(0),
        "lists": sqlx::query_scalar::<_, Option<i64>>("SELECT COUNT(*) FROM lists").fetch_one(pool).await.unwrap_or(None).unwrap_or(0),
        "users": sqlx::query_scalar::<_, Option<i64>>("SELECT COUNT(*) FROM users").fetch_one(pool).await.unwrap_or(None).unwrap_or(0),
        "comments": sqlx::query_scalar::<_, Option<i64>>("SELECT COUNT(*) FROM comments").fetch_one(pool).await.unwrap_or(None).unwrap_or(0),
    })
}

async fn health(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
=======
async fn health() -> Json<serde_json::Value> {
>>>>>>> origin/main
    let started = START_TIME.load(Ordering::Relaxed);
    let uptime = if started > 0 { epoch_secs().saturating_sub(started) } else { 0 };

    Json(json!({
        "status": "ok",
        "memory": read_memory_kb(),
        "uptime_seconds": uptime,
<<<<<<< HEAD
        "rust_version": env!("CARGO_PKG_RUST_VERSION"),
        "db_stats": db_stats(&state.db).await,
=======
>>>>>>> origin/main
    }))
}
