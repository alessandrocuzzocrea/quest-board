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

    let state = Arc::new(AppState { db: pool.clone(), ai_client: Arc::new(quest_board::handlers::ai::RealLlmClient) });
    let app = quest_board::build_app(pool.clone(), state).await;
    TestApp { _guard: guard, app, _pool: pool }
}

/// Register a test user and return the session cookie.
async fn register(app: &axum::Router, name: &str) -> String {
    let body = format!(r#"{{"username":"{name}","password":"pass","name":"{name} Tester"}}"#);
    let req = axum::http::Request::builder()
        .method("POST").uri("/api/v1/auth/register")
        .header("content-type", "application/json")
        .body(axum::body::Body::from(body)).unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);
    resp.headers().get("set-cookie")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.split(';').next().unwrap_or("").to_string())
        .unwrap()
}

/// Create a board and return its id.
async fn create_board(app: &axum::Router, cookie: &str) -> String {
    let req = axum::http::Request::builder()
        .method("POST").uri("/api/v1/boards")
        .header("content-type", "application/json").header("cookie", cookie)
        .body(axum::body::Body::from(r#"{"name":"Label Test Board"}"#)).unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    v["id"].as_str().unwrap().to_string()
}

/// Create a label on a board, return the JSON response.
async fn create_label(app: &axum::Router, cookie: &str, board_id: &str, name: &str, color: Option<&str>) -> serde_json::Value {
    let color_part = color.map(|c| format!(r#","color":"{c}""#)).unwrap_or_default();
    let body = format!(r#"{{"board_id":"{board_id}","name":"{name}"{color_part}}}"#);
    let req = axum::http::Request::builder()
        .method("POST").uri("/api/v1/labels")
        .header("content-type", "application/json").header("cookie", cookie)
        .body(axum::body::Body::from(body)).unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    let status = resp.status();
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    if status != 200 {
        let text = String::from_utf8_lossy(&bytes);
        panic!("create_label {name} failed ({status}): {text}");
    }
    serde_json::from_slice(&bytes).unwrap()
}

// ── Tests ──────────────────────────────────────────────────────

#[tokio::test]
async fn test_create_label_requires_auth() {
    let ta = setup().await;

    let req = axum::http::Request::builder()
        .method("POST").uri("/api/v1/labels")
        .header("content-type", "application/json")
        .body(axum::body::Body::from(r#"{"board_id":"00000000-0000-0000-0000-000000000000","name":"test"}"#)).unwrap();
    let resp = ta.app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 401, "label creation without auth should be rejected");
}

#[tokio::test]
async fn test_create_and_list_labels() {
    let ta = setup().await;
    let cookie = register(&ta.app, "labeluser1").await;
    let board_id = create_board(&ta.app, &cookie).await;

    // Create two labels
    let l1 = create_label(&ta.app, &cookie, &board_id, "bug", Some("#ff0000")).await;
    let l2 = create_label(&ta.app, &cookie, &board_id, "feature", Some("#00ff00")).await;

    assert_eq!(l1["name"].as_str(), Some("bug"));
    assert_eq!(l1["color"].as_str(), Some("#ff0000"));
    assert_eq!(l2["name"].as_str(), Some("feature"));
    assert_eq!(l2["color"].as_str(), Some("#00ff00"));

    // List labels on the board
    let req = axum::http::Request::builder()
        .method("GET").uri(&format!("/api/v1/labels/board/{board_id}"))
        .header("cookie", &cookie)
        .body(axum::body::Body::empty()).unwrap();
    let resp = ta.app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let labels: Vec<serde_json::Value> = serde_json::from_slice(&bytes).unwrap();

    assert_eq!(labels.len(), 2);
    assert!(labels.iter().any(|l| l["name"] == "bug"));
    assert!(labels.iter().any(|l| l["name"] == "feature"));
}

#[tokio::test]
async fn test_create_label_defaults_color() {
    let ta = setup().await;
    let cookie = register(&ta.app, "labeluser2").await;
    let board_id = create_board(&ta.app, &cookie).await;

    // Create label without specifying color — should use default
    let l = create_label(&ta.app, &cookie, &board_id, "default-color", None).await;
    assert_eq!(l["name"].as_str(), Some("default-color"));
    // Default color is #0079bf (from the handler)
    assert_eq!(l["color"].as_str(), Some("#0079bf"));
}

#[tokio::test]
async fn test_labels_scoped_to_board() {
    let ta = setup().await;
    let cookie = register(&ta.app, "labeluser3").await;
    let board1 = create_board(&ta.app, &cookie).await;
    let board2 = create_board(&ta.app, &cookie).await;

    create_label(&ta.app, &cookie, &board1, "board1-label", None).await;

    // Board2 should have no labels
    let req = axum::http::Request::builder()
        .method("GET").uri(&format!("/api/v1/labels/board/{board2}"))
        .header("cookie", &cookie)
        .body(axum::body::Body::empty()).unwrap();
    let resp = ta.app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let labels: Vec<serde_json::Value> = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(labels.len(), 0, "board2 should have no labels");
}

#[tokio::test]
async fn test_update_label() {
    let ta = setup().await;
    let cookie = register(&ta.app, "labeluser4").await;
    let board_id = create_board(&ta.app, &cookie).await;
    let label = create_label(&ta.app, &cookie, &board_id, "old-name", Some("#ff0000")).await;
    let label_id = label["id"].as_str().unwrap().to_string();

    let body = br##"{"name":"new-name","color":"#0000ff"}"##;
    let req = axum::http::Request::builder()
        .method("PUT").uri(&format!("/api/v1/labels/{label_id}"))
        .header("content-type", "application/json").header("cookie", &cookie)
        .body(axum::body::Body::from(body.to_vec())).unwrap();
    let resp = ta.app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let updated: serde_json::Value = serde_json::from_slice(&bytes).unwrap();

    assert_eq!(updated["name"].as_str(), Some("new-name"));
    assert_eq!(updated["color"].as_str(), Some("#0000ff"));
}

#[tokio::test]
async fn test_delete_label() {
    let ta = setup().await;
    let cookie = register(&ta.app, "labeluser5").await;
    let board_id = create_board(&ta.app, &cookie).await;

    let label = create_label(&ta.app, &cookie, &board_id, "to-delete", None).await;
    let label_id = label["id"].as_str().unwrap().to_string();

    // Delete
    let req = axum::http::Request::builder()
        .method("DELETE").uri(&format!("/api/v1/labels/{label_id}"))
        .header("cookie", &cookie)
        .body(axum::body::Body::empty()).unwrap();
    let resp = ta.app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);

    // Verify it's gone
    let req = axum::http::Request::builder()
        .method("GET").uri(&format!("/api/v1/labels/board/{board_id}"))
        .header("cookie", &cookie)
        .body(axum::body::Body::empty()).unwrap();
    let resp = ta.app.clone().oneshot(req).await.unwrap();
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let labels: Vec<serde_json::Value> = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(labels.len(), 0, "label should be deleted");
}

#[tokio::test]
async fn test_delete_label_requires_auth() {
    let ta = setup().await;

    let req = axum::http::Request::builder()
        .method("DELETE").uri("/api/v1/labels/00000000-0000-0000-0000-000000000000")
        .body(axum::body::Body::empty()).unwrap();
    let resp = ta.app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 401, "label deletion without auth should be rejected");
}

#[tokio::test]
async fn test_list_labels_requires_auth() {
    let ta = setup().await;

    let req = axum::http::Request::builder()
        .method("GET").uri("/api/v1/labels/board/00000000-0000-0000-0000-000000000000")
        .body(axum::body::Body::empty()).unwrap();
    let resp = ta.app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 401);
}
