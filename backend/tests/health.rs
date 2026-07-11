use quest_board::AppState;
use std::sync::Arc;
use tower::ServiceExt;

async fn setup() -> axum::Router {
    dotenvy::from_filename("../backend/.env.test").ok();
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:quest@localhost:5432/quest_test".into());
    let pool = sqlx::PgPool::connect(&database_url).await.unwrap();
    let state = Arc::new(AppState {
        db: pool.clone(),
        ai_client: Arc::new(quest_board::handlers::ai::RealLlmClient),
    });
    quest_board::build_app(pool.clone(), state).await
}

#[tokio::test]
async fn test_health_returns_200() {
    let app = setup().await;
    let req = axum::http::Request::builder()
        .method("GET").uri("/api/v1/health")
        .body(axum::body::Body::empty()).unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);
}

#[tokio::test]
async fn test_health_contains_memory_and_uptime() {
    let app = setup().await;
    let req = axum::http::Request::builder()
        .method("GET").uri("/api/v1/health")
        .body(axum::body::Body::empty()).unwrap();
    let resp = app.oneshot(req).await.unwrap();
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert!(json["memory"].as_str().unwrap_or("").contains("kB"), "expected memory in kB, got {:?}", json["memory"]);
    assert!(json.get("uptime_seconds").is_some(), "expected uptime_seconds field");
}
