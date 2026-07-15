use quest_board::AppState;
use std::sync::Arc;
use std::sync::LazyLock;
use tokio::sync::Mutex;
use tokio::sync::MutexGuard;
use tower::ServiceExt;

static SETUP_MUTEX: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

struct TestApp {
    _guard: MutexGuard<'static, ()>,
    app: axum::Router,
}

async fn setup() -> TestApp {
    let _guard = SETUP_MUTEX.lock().await;
    dotenvy::from_filename(".env.test").ok();
    let db = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:quest@localhost:5432/quest_test".into());
    let pool = sqlx::PgPool::connect(&db).await.unwrap();
    sqlx::query("DROP SCHEMA public CASCADE").execute(&pool).await.ok();
    sqlx::query("CREATE SCHEMA public").execute(&pool).await.ok();
    sqlx::query("GRANT ALL ON SCHEMA public TO postgres").execute(&pool).await.ok();
    sqlx::query("GRANT ALL ON SCHEMA public TO public").execute(&pool).await.ok();
    quest_board::db::run_migrations(&pool).await.unwrap();
    let (event_tx, _) = quest_board::events::channel();
    let state = Arc::new(AppState {
        db: pool.clone(),
        event_tx,
    });
    let app = quest_board::build_app(pool.clone(), state).await;
    TestApp { _guard, app }
}

#[tokio::test]
async fn test_health_returns_200() {
    let ta = setup().await;
    let req = axum::http::Request::builder()
        .method("GET").uri("/api/v1/health")
        .body(axum::body::Body::empty()).unwrap();
    let resp = ta.app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);
}

#[tokio::test]
async fn test_health_contains_system_info() {
    let ta = setup().await;
    let req = axum::http::Request::builder()
        .method("GET").uri("/api/v1/health")
        .body(axum::body::Body::empty()).unwrap();
    let resp = ta.app.oneshot(req).await.unwrap();
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert!(json["memory"].as_str().unwrap_or("").contains("kB"));
    assert!(json.get("uptime_seconds").is_some());
    assert!(json.get("rust_version").is_some());
    assert!(json.get("db_stats").is_some());
    assert!(json["db_stats"]["boards"].is_number());
    assert!(json["db_stats"]["cards"].is_number());
    assert!(json["db_stats"]["users"].is_number());
}
