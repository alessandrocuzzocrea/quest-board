use quest_board::AppState;
use std::sync::Arc;
use std::sync::LazyLock;
use tokio::sync::{Mutex, MutexGuard};
use tower::ServiceExt;

static SETUP_MUTEX: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

struct TestApp {
    _guard: MutexGuard<'static, ()>,
    app: axum::Router,
    _pool: sqlx::PgPool,
}

async fn setup() -> TestApp {
    let _guard = SETUP_MUTEX.lock().await;
    dotenvy::from_filename(".env.test").ok();
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:quest@localhost:5432/quest_test".into());
    let pool = sqlx::PgPool::connect(&database_url).await.unwrap();
    sqlx::query("DROP TABLE IF EXISTS api_keys, sessions, favorites, notifications, actions, tasks, task_lists, attachments, comments, card_labels, labels, card_members, cards, lists, board_members, boards, users CASCADE")
        .execute(&pool).await.ok();
    quest_board::db::run_migrations(&pool).await.unwrap();
    let state = Arc::new(AppState { db: pool.clone(), ai_client: Arc::new(quest_board::handlers::ai::RealLlmClient) });
    let app = quest_board::build_app(pool.clone(), state).await;
    TestApp { _guard, app, _pool: pool }
}

#[tokio::test]
async fn test_create_comment_requires_auth() {
    let ta = setup().await;
    let req = axum::http::Request::builder()
        .method("POST").uri("/api/v1/comments")
        .header("content-type", "application/json")
        .body(axum::body::Body::from(r#"{"card_id":"00000000-0000-0000-0000-000000000000","text":"hi"}"#)).unwrap();
    let resp = ta.app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 401);
}

#[tokio::test]
async fn test_create_and_list_comment() {
    let ta = setup().await;

    let req = axum::http::Request::builder()
        .method("POST").uri("/api/v1/auth/register")
        .header("content-type", "application/json")
        .body(axum::body::Body::from(r#"{"username":"c","password":"p","name":"T"}"#)).unwrap();
    let resp = ta.app.clone().oneshot(req).await.unwrap();
    let cookie = resp.headers().get("set-cookie").and_then(|v| v.to_str().ok())
        .map(|s| s.split(';').next().unwrap_or("").to_string()).unwrap();

    let req = axum::http::Request::builder()
        .method("POST").uri("/api/v1/boards")
        .header("content-type", "application/json").header("cookie", &cookie)
        .body(axum::body::Body::from(r#"{"name":"Board"}"#)).unwrap();
    let resp = ta.app.clone().oneshot(req).await.unwrap();
    let board: serde_json::Value = serde_json::from_slice(
        &axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap()
    ).unwrap();
    let board_id = board["id"].as_str().unwrap();

    let req = axum::http::Request::builder()
        .method("POST").uri("/api/v1/lists")
        .header("content-type", "application/json").header("cookie", &cookie)
        .body(axum::body::Body::from(
            format!(r#"{{"board_id":"{board_id}","name":"List"}}"#)
        )).unwrap();
    let resp = ta.app.clone().oneshot(req).await.unwrap();
    let list: serde_json::Value = serde_json::from_slice(
        &axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap()
    ).unwrap();
    let list_id = list["id"].as_str().unwrap();

    let req = axum::http::Request::builder()
        .method("POST").uri("/api/v1/cards")
        .header("content-type", "application/json").header("cookie", &cookie)
        .body(axum::body::Body::from(
            format!(r#"{{"list_id":"{list_id}","name":"Card"}}"#)
        )).unwrap();
    let resp = ta.app.clone().oneshot(req).await.unwrap();
    let card: serde_json::Value = serde_json::from_slice(
        &axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap()
    ).unwrap();
    let card_id = card["id"].as_str().unwrap();

    let req = axum::http::Request::builder()
        .method("POST").uri("/api/v1/comments")
        .header("content-type", "application/json").header("cookie", &cookie)
        .body(axum::body::Body::from(
            format!(r#"{{"card_id":"{card_id}","text":"Hello"}}"#)
        )).unwrap();
    let resp = ta.app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);
    let comment: serde_json::Value = serde_json::from_slice(
        &axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap()
    ).unwrap();
    assert_eq!(comment["text"], "Hello");
}

// ── Helper functions ──────────────────────────────────────────

async fn register(app: &axum::Router) -> String {
    let req = axum::http::Request::builder()
        .method("POST").uri("/api/v1/auth/register")
        .header("content-type", "application/json")
        .body(axum::body::Body::from(r#"{"username":"cu","password":"p","name":"CommentUser"}"#)).unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);
    resp.headers().get("set-cookie").and_then(|v| v.to_str().ok())
        .map(|s| s.split(';').next().unwrap_or("").to_string()).unwrap()
}

async fn create_board(app: &axum::Router, cookie: &str) -> String {
    let req = axum::http::Request::builder()
        .method("POST").uri("/api/v1/boards")
        .header("content-type", "application/json").header("cookie", cookie)
        .body(axum::body::Body::from(r#"{"name":"Comment Board"}"#)).unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    v["id"].as_str().unwrap().to_string()
}

async fn create_list(app: &axum::Router, cookie: &str, board_id: &str) -> String {
    let body = format!(r#"{{"board_id":"{board_id}","name":"List"}}"#);
    let req = axum::http::Request::builder()
        .method("POST").uri("/api/v1/lists")
        .header("content-type", "application/json").header("cookie", cookie)
        .body(axum::body::Body::from(body)).unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    v["id"].as_str().unwrap().to_string()
}

async fn create_card(app: &axum::Router, cookie: &str, list_id: &str) -> String {
    let body = format!(r#"{{"list_id":"{list_id}","name":"Card"}}"#);
    let req = axum::http::Request::builder()
        .method("POST").uri("/api/v1/cards")
        .header("content-type", "application/json").header("cookie", cookie)
        .body(axum::body::Body::from(body)).unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    v["id"].as_str().unwrap().to_string()
}

async fn create_comment(app: &axum::Router, cookie: &str, card_id: &str, text: &str) -> serde_json::Value {
    let body = format!(r#"{{"card_id":"{card_id}","text":"{text}"}}"#);
    let req = axum::http::Request::builder()
        .method("POST").uri("/api/v1/comments")
        .header("content-type", "application/json").header("cookie", cookie)
        .body(axum::body::Body::from(body)).unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

// ── Update / Delete tests ─────────────────────────────────────

#[tokio::test]
async fn test_update_comment() {
    let ta = setup().await;
    let cookie = register(&ta.app).await;
    let board_id = create_board(&ta.app, &cookie).await;
    let list_id = create_list(&ta.app, &cookie, &board_id).await;
    let card_id = create_card(&ta.app, &cookie, &list_id).await;
    let comment = create_comment(&ta.app, &cookie, &card_id, "Original text").await;
    let comment_id = comment["id"].as_str().unwrap().to_string();
    assert_eq!(comment["text"].as_str(), Some("Original text"));

    // Update the comment text
    let body = r#"{"text":"Updated text"}"#;
    let req = axum::http::Request::builder()
        .method("PUT").uri(&format!("/api/v1/comments/{comment_id}"))
        .header("content-type", "application/json").header("cookie", &cookie)
        .body(axum::body::Body::from(body)).unwrap();
    let resp = ta.app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let updated: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(updated["text"].as_str(), Some("Updated text"));
}

#[tokio::test]
async fn test_update_comment_requires_auth() {
    let ta = setup().await;
    let req = axum::http::Request::builder()
        .method("PUT").uri("/api/v1/comments/00000000-0000-0000-0000-000000000000")
        .header("content-type", "application/json")
        .body(axum::body::Body::from(r#"{"text":"x"}"#)).unwrap();
    let resp = ta.app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 401);
}

#[tokio::test]
async fn test_delete_comment() {
    let ta = setup().await;
    let cookie = register(&ta.app).await;
    let board_id = create_board(&ta.app, &cookie).await;
    let list_id = create_list(&ta.app, &cookie, &board_id).await;
    let card_id = create_card(&ta.app, &cookie, &list_id).await;
    let comment = create_comment(&ta.app, &cookie, &card_id, "To delete").await;
    let comment_id = comment["id"].as_str().unwrap().to_string();

    // Delete
    let req = axum::http::Request::builder()
        .method("DELETE").uri(&format!("/api/v1/comments/{comment_id}"))
        .header("cookie", &cookie)
        .body(axum::body::Body::empty()).unwrap();
    let resp = ta.app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);

    // Verify via GET /cards/{card_id}/comments
    let req = axum::http::Request::builder()
        .method("GET").uri(&format!("/api/v1/cards/{card_id}/comments"))
        .header("cookie", &cookie)
        .body(axum::body::Body::empty()).unwrap();
    let resp = ta.app.oneshot(req).await.unwrap();
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let list: Vec<serde_json::Value> = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(list.len(), 0, "comment should be deleted");
}

#[tokio::test]
async fn test_delete_comment_requires_auth() {
    let ta = setup().await;
    let req = axum::http::Request::builder()
        .method("DELETE").uri("/api/v1/comments/00000000-0000-0000-0000-000000000000")
        .body(axum::body::Body::empty()).unwrap();
    let resp = ta.app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 401);
}
