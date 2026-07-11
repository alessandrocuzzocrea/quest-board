use quest_board::AppState;
use std::sync::Arc;
use std::sync::LazyLock;
use tokio::sync::Mutex;
use tower::ServiceExt;

static SETUP_MUTEX: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

async fn setup() -> (axum::Router, sqlx::PgPool) {
    let _guard = SETUP_MUTEX.lock().await;
    dotenvy::from_filename("../backend/.env.test").ok();
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:quest@localhost:5432/quest_test".into());
    let pool = sqlx::PgPool::connect(&database_url).await.unwrap();
    sqlx::query("DROP TABLE IF EXISTS api_keys, sessions, favorites, notifications, actions, tasks, task_lists, attachments, comments, card_labels, labels, card_members, cards, lists, board_members, boards, users CASCADE")
        .execute(&pool).await.ok();
    quest_board::db::run_migrations(&pool).await.unwrap();
    let state = Arc::new(AppState { db: pool.clone(), ai_client: Arc::new(quest_board::handlers::ai::RealLlmClient) });
    let app = quest_board::build_app(pool.clone(), state).await;
    (app, pool)
}

#[tokio::test]
async fn test_create_comment_requires_auth() {
    let (app, _pool) = setup().await;
    let req = axum::http::Request::builder()
        .method("POST").uri("/api/v1/comments")
        .header("content-type", "application/json")
        .body(axum::body::Body::from(r#"{"card_id":"00000000-0000-0000-0000-000000000000","text":"hi"}"#)).unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 401);
}

#[tokio::test]
async fn test_create_and_list_comment() {
    let (app, _pool) = setup().await;

    let req = axum::http::Request::builder()
        .method("POST").uri("/api/v1/auth/register")
        .header("content-type", "application/json")
        .body(axum::body::Body::from(r#"{"email":"c@test.com","password":"p","name":"T"}"#)).unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    let cookie = resp.headers().get("set-cookie").and_then(|v| v.to_str().ok())
        .map(|s| s.split(';').next().unwrap_or("").to_string()).unwrap();

    let req = axum::http::Request::builder()
        .method("POST").uri("/api/v1/boards")
        .header("content-type", "application/json").header("cookie", &cookie)
        .body(axum::body::Body::from(r#"{"name":"Board"}"#)).unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    let board: serde_json::Value = serde_json::from_slice(
        &axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap()
    ).unwrap();
    let board_id = board["id"].as_str().unwrap();

    let req = axum::http::Request::builder()
        .method("GET").uri(&format!("/api/v1/boards/{board_id}"))
        .header("cookie", &cookie)
        .body(axum::body::Body::empty()).unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    let bd: serde_json::Value = serde_json::from_slice(
        &axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap()
    ).unwrap();
    let list_id = bd["lists"][0]["id"].as_str().unwrap();

    let body = format!(r#"{{"list_id":"{list_id}","name":"Card"}}"#);
    let req = axum::http::Request::builder()
        .method("POST").uri("/api/v1/cards")
        .header("content-type", "application/json").header("cookie", &cookie)
        .body(axum::body::Body::from(body)).unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    let card: serde_json::Value = serde_json::from_slice(
        &axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap()
    ).unwrap();
    let card_id = card["id"].as_str().unwrap();

    let body = format!(r#"{{"card_id":"{card_id}","text":"Hello!"}}"#);
    let req = axum::http::Request::builder()
        .method("POST").uri("/api/v1/comments")
        .header("content-type", "application/json").header("cookie", &cookie)
        .body(axum::body::Body::from(body)).unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200, "create comment");

    let req = axum::http::Request::builder()
        .method("GET").uri(&format!("/api/v1/cards/{card_id}/comments"))
        .header("cookie", &cookie)
        .body(axum::body::Body::empty()).unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);
    let comments: Vec<serde_json::Value> = serde_json::from_slice(
        &axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap()
    ).unwrap();
    assert_eq!(comments.len(), 1);
    assert_eq!(comments[0]["text"], "Hello!");
}
