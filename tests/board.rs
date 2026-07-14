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
