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
    _pool: sqlx::PgPool,
}


async fn setup() -> TestApp {
    let guard = SETUP_MUTEX.lock().await;
    dotenvy::from_filename(".env.test").ok();

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:quest@localhost:5432/quest_test".into());

    let pool = sqlx::PgPool::connect(&database_url)
        .await
        .expect("failed to connect");

    sqlx::query("DROP TABLE IF EXISTS sessions, favorites, notifications, actions, tasks, task_lists, attachments, comments, card_labels, labels, card_members, cards, lists, board_members, boards, users CASCADE")
        .execute(&pool).await.ok();

    quest_board::db::run_migrations(&pool)
        .await
        .expect("failed to run migrations");

    let (event_tx, _) = quest_board::events::channel();
    let state = Arc::new(AppState { db: pool.clone(), ai_client: Arc::new(quest_board::handlers::ai::RealLlmClient), event_tx });
    let app = quest_board::build_app(pool.clone(), state).await;
    TestApp { _guard: guard, app, _pool: pool }
}

async fn register(app: &axum::Router, username: &str) -> String {
    let body = format!(r#"{{"username":"{username}","password":"pass","name":"T"}}"#);
    let req = axum::http::Request::builder()
        .method("POST").uri("/api/v1/auth/register")
        .header("content-type", "application/json")
        .body(axum::body::Body::from(body)).unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);
    resp.headers().get("set-cookie").and_then(|v| v.to_str().ok())
        .map(|s| s.split(';').next().unwrap_or("").to_string()).unwrap()
}

#[tokio::test]
async fn test_create_board_requires_auth() {
    let ta = setup().await;
    let req = axum::http::Request::builder()
        .method("POST").uri("/api/v1/boards")
        .header("content-type", "application/json")
        .body(axum::body::Body::from(r#"{"name":"Test"}"#)).unwrap();
    let resp = ta.app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 401, "unauthenticated POST should be 401, got {}", resp.status());
}

#[tokio::test]
async fn test_create_and_list_board() {
    let ta = setup().await;
    let cookie = register(&ta.app, "testuser").await;

    // Create board
    let req = axum::http::Request::builder()
        .method("POST").uri("/api/v1/boards")
        .header("content-type", "application/json")
        .header("cookie", &cookie)
        .body(axum::body::Body::from(r#"{"name":"Test Board"}"#)).unwrap();
    let resp = ta.app.clone().oneshot(req).await.unwrap();
    let status = resp.status();
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    assert_eq!(status, 200, "create board: {}", String::from_utf8_lossy(&bytes));
    let board: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(board["name"], "Test Board");
    assert!(board["slug"].as_str().unwrap_or("").len() >= 6, "no slug");

    // List boards
    let req = axum::http::Request::builder()
        .method("GET").uri("/api/v1/boards")
        .header("cookie", &cookie)
        .body(axum::body::Body::empty()).unwrap();
    let resp = ta.app.clone().oneshot(req).await.unwrap();
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let boards: Vec<serde_json::Value> = serde_json::from_slice(&bytes).unwrap_or_default();
    assert_eq!(boards.len(), 1);
    assert_eq!(boards[0]["name"], "Test Board");
}

#[tokio::test]
async fn test_board_not_visible_to_other_users() {
    let ta = setup().await;
    let cookie_a = register(&ta.app, "alice").await;
    let cookie_b = register(&ta.app, "bob").await;

    // Alice creates board
    let req = axum::http::Request::builder()
        .method("POST").uri("/api/v1/boards")
        .header("content-type", "application/json")
        .header("cookie", &cookie_a)
        .body(axum::body::Body::from(r#"{"name":"Alice's Board"}"#)).unwrap();
    let resp = ta.app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200, "alice create board");

    // Bob lists boards
    let req = axum::http::Request::builder()
        .method("GET").uri("/api/v1/boards")
        .header("cookie", &cookie_b)
        .body(axum::body::Body::empty()).unwrap();
    let resp = ta.app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200, "bob list boards");
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let boards: Vec<serde_json::Value> = serde_json::from_slice(&bytes).unwrap_or_default();
    assert_eq!(boards.len(), 0, "bob should not see alice's board");
}

#[tokio::test]
async fn test_board_slug_survives_migration_reapply() {
    let ta = setup().await;
    let cookie = register(&ta.app, "slugtest").await;

    // Create board
    let req = axum::http::Request::builder()
        .method("POST").uri("/api/v1/boards")
        .header("content-type", "application/json")
        .header("cookie", &cookie)
        .body(axum::body::Body::from(r#"{"name":"Slug Test"}"#)).unwrap();
    let resp = ta.app.clone().oneshot(req).await.unwrap();
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let board: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    let original_slug = board["slug"].as_str().unwrap().to_string();
    assert!(original_slug.len() >= 6, "board should have a slug");

    // Re-run migrations (simulating server restart)
    quest_board::db::run_migrations(&ta._pool)
        .await
        .expect("migrations re-apply should succeed");

    // Fetch board by slug — should still work
    let req = axum::http::Request::builder()
        .method("GET").uri(&format!("/api/v1/boards/by-slug/{}", original_slug))
        .header("cookie", &cookie)
        .body(axum::body::Body::empty()).unwrap();
    let resp = ta.app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200, "board should still be found by original slug after migration re-apply");
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let data: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    let slug_after = data["board"]["slug"].as_str().unwrap();
    assert_eq!(slug_after, original_slug, "slug must not change when migrations re-run");
}

#[tokio::test]
async fn test_search_returns_html_for_htmx_requests() {
    let ta = setup().await;
    let cookie = register(&ta.app, "searchh").await;

    // Create a couple boards with different names
    for name in ["Alpha Board", "Beta Project"] {
        let req = axum::http::Request::builder()
            .method("POST").uri("/api/v1/boards")
            .header("content-type", "application/json")
            .header("cookie", &cookie)
            .body(axum::body::Body::from(format!(r#"{{"name":"{name}"}}"#))).unwrap();
        let resp = ta.app.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), 200, "create {name}");
    }
    // HTMX search: expect HTML partial
    let req = axum::http::Request::builder()
        .method("GET").uri("/api/v1/search?q=Alpha")
        .header("cookie", &cookie)
        .header("HX-Request", "true")
        .body(axum::body::Body::empty()).unwrap();
    let resp = ta.app.clone().oneshot(req).await.unwrap();
    let status = resp.status();
    let content_type = resp.headers().get("content-type").and_then(|v| v.to_str().ok()).unwrap_or("").to_string();
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    assert_eq!(status, 200, "HTMX search: {}", String::from_utf8_lossy(&bytes));
    assert!(content_type.starts_with("text/html"), "HTMX search should return HTML, got: {content_type}");
    let html = String::from_utf8_lossy(&bytes);
    assert!(html.contains("Alpha Board"), "HTMX response should contain matching board name");
    assert!(!html.contains("Beta Project"), "HTMX response should not contain non-matching boards");

    // JSON search (no HX-Request): expect JSON
    let req = axum::http::Request::builder()
        .method("GET").uri("/api/v1/search?q=Alpha")
        .header("cookie", &cookie)
        .body(axum::body::Body::empty()).unwrap();
    let resp = ta.app.clone().oneshot(req).await.unwrap();
    let status = resp.status();
    let content_type = resp.headers().get("content-type").and_then(|v| v.to_str().ok()).unwrap_or("").to_string();
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    assert_eq!(status, 200, "JSON search: {}", String::from_utf8_lossy(&bytes));
    assert!(content_type.starts_with("application/json"), "JSON search should return JSON, got: {content_type}");
    let data: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    let board_names: Vec<&str> = data["boards"].as_array().unwrap().iter()
        .filter_map(|b| b["name"].as_str()).collect();
    assert!(board_names.contains(&"Alpha Board"), "JSON should contain matching board");
    assert!(!board_names.contains(&"Beta Project"), "JSON should not contain non-matching board");

    // Empty query returns all boards
    let req = axum::http::Request::builder()
        .method("GET").uri("/api/v1/search?q=")
        .header("cookie", &cookie)
        .header("HX-Request", "true")
        .body(axum::body::Body::empty()).unwrap();
    let resp = ta.app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200, "empty query search should succeed");
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let html = String::from_utf8_lossy(&bytes);
    assert!(html.contains("Alpha Board"), "empty query should return all boards");
    assert!(html.contains("Beta Project"), "empty query should return all boards");
}

#[tokio::test]
async fn test_board_service_list_accessible() {
    let ta = setup().await;
    let cookie = register(&ta.app, "svctest").await;

    // Get the user's UUID from DB
    let user_id: uuid::Uuid = sqlx::query_scalar("SELECT id FROM users WHERE username = $1")
        .bind("svctest")
        .fetch_one(&ta._pool)
        .await
        .expect("user should exist");

    // Service should initially return no boards
    let svc = quest_board::services::BoardService::new(ta._pool.clone());
    let boards = svc.list_accessible(&user_id).await.unwrap();
    assert!(boards.is_empty(), "new user should have no boards");

    // Create a board via API
    let req = axum::http::Request::builder()
        .method("POST").uri("/api/v1/boards")
        .header("content-type", "application/json")
        .header("cookie", &cookie)
        .body(axum::body::Body::from(r#"{"name":"Service Board"}"#)).unwrap();
    let resp = ta.app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);

    // Service should now return the board
    let boards = svc.list_accessible(&user_id).await.unwrap();
    assert_eq!(boards.len(), 1);
    assert_eq!(boards[0].name, "Service Board");

    // A different user should not see the board
    let other_cookie = register(&ta.app, "otheruser").await;
    let other_id: uuid::Uuid = sqlx::query_scalar("SELECT id FROM users WHERE username = $1")
        .bind("otheruser")
        .fetch_one(&ta._pool)
        .await
        .expect("other user should exist");
    let other_boards = svc.list_accessible(&other_id).await.unwrap();
    assert!(other_boards.is_empty(), "other user should not see the board");
}
