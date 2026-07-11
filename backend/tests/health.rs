use quest_board::AppState;
use std::sync::Arc;
use tower::ServiceExt;

async fn setup() -> axum::Router {
    dotenvy::from_filename("../backend/.env.test").ok();
    let db = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:quest@localhost:5432/quest_test".into());
    let pool = sqlx::PgPool::connect(&db).await.unwrap();
    sqlx::query("DROP TABLE IF EXISTS api_keys,sessions,favorites,notifications,actions,tasks,task_lists,attachments,comments,card_labels,labels,card_members,cards,lists,board_members,boards,users CASCADE")
        .execute(&pool).await.ok();
    quest_board::db::run_migrations(&pool).await.unwrap();
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
async fn test_health_contains_system_info() {
    let app = setup().await;
    let req = axum::http::Request::builder()
        .method("GET").uri("/api/v1/health")
        .body(axum::body::Body::empty()).unwrap();
    let resp = app.oneshot(req).await.unwrap();
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
