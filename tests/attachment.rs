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

// ── Helpers ────────────────────────────────────────────────────

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

async fn create_board(app: &axum::Router, cookie: &str) -> String {
    let req = axum::http::Request::builder()
        .method("POST").uri("/api/v1/boards")
        .header("content-type", "application/json").header("cookie", cookie)
        .body(axum::body::Body::from(r#"{"name":"Attachment Test Board"}"#)).unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    v["id"].as_str().unwrap().to_string()
}

async fn create_list(app: &axum::Router, cookie: &str, board_id: &str) -> String {
    let body = format!(r#"{{"board_id":"{board_id}","name":"Test List"}}"#);
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
    let body = format!(r#"{{"list_id":"{list_id}","name":"Attachment Test Card"}}"#);
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

/// POST /attachments/link — create a link attachment.
async fn create_link_attachment(app: &axum::Router, cookie: &str, card_id: &str, name: &str, url: &str) -> serde_json::Value {
    let body = format!(r#"{{"card_id":"{card_id}","name":"{name}","url":"{url}"}}"#);
    let req = axum::http::Request::builder()
        .method("POST").uri("/api/v1/attachments/link")
        .header("content-type", "application/json").header("cookie", cookie)
        .body(axum::body::Body::from(body)).unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    let status = resp.status();
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    if status != 200 {
        let text = String::from_utf8_lossy(&bytes);
        panic!("create_link_attachment failed ({status}): {text}");
    }
    serde_json::from_slice(&bytes).unwrap()
}

// ── Tests ──────────────────────────────────────────────────────

#[tokio::test]
async fn test_create_link_requires_auth() {
    let ta = setup().await;

    let req = axum::http::Request::builder()
        .method("POST").uri("/api/v1/attachments/link")
        .header("content-type", "application/json")
        .body(axum::body::Body::from(r#"{"card_id":"00000000-0000-0000-0000-000000000000","url":"https://example.com"}"#)).unwrap();
    let resp = ta.app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 401, "attachment creation without auth should be rejected");
}

#[tokio::test]
async fn test_list_attachments_requires_auth() {
    let ta = setup().await;

    let req = axum::http::Request::builder()
        .method("GET").uri("/api/v1/attachments/card/00000000-0000-0000-0000-000000000000")
        .body(axum::body::Body::empty()).unwrap();
    let resp = ta.app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 401);
}

#[tokio::test]
async fn test_create_and_list_link_attachments() {
    let ta = setup().await;
    let cookie = register(&ta.app, "attuser1").await;
    let board_id = create_board(&ta.app, &cookie).await;
    let list_id = create_list(&ta.app, &cookie, &board_id).await;
    let card_id = create_card(&ta.app, &cookie, &list_id).await;

    // Create a link attachment
    let a1 = create_link_attachment(&ta.app, &cookie, &card_id, "GitHub", "https://github.com").await;
    assert_eq!(a1["name"].as_str(), Some("GitHub"));
    assert_eq!(a1["link_url"].as_str(), Some("https://github.com"));
    assert_eq!(a1["type"].as_str(), Some("link"), "attachment type should be 'link'");

    // Create another
    let a2 = create_link_attachment(&ta.app, &cookie, &card_id, "Docs", "https://docs.example.com").await;
    assert_eq!(a2["name"].as_str(), Some("Docs"));

    // List attachments on the card
    let req = axum::http::Request::builder()
        .method("GET").uri(&format!("/api/v1/attachments/card/{card_id}"))
        .header("cookie", &cookie)
        .body(axum::body::Body::empty()).unwrap();
    let resp = ta.app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let list: Vec<serde_json::Value> = serde_json::from_slice(&bytes).unwrap();

    assert_eq!(list.len(), 2);
    assert!(list.iter().any(|a| a["name"] == "GitHub"));
    assert!(list.iter().any(|a| a["name"] == "Docs"));
}

#[tokio::test]
async fn test_create_link_missing_url_fails() {
    let ta = setup().await;
    let cookie = register(&ta.app, "attuser2").await;
    let board_id = create_board(&ta.app, &cookie).await;
    let list_id = create_list(&ta.app, &cookie, &board_id).await;
    let card_id = create_card(&ta.app, &cookie, &list_id).await;

    let body = format!(r#"{{"card_id":"{card_id}","name":"No URL"}}"#);
    let req = axum::http::Request::builder()
        .method("POST").uri("/api/v1/attachments/link")
        .header("content-type", "application/json").header("cookie", &cookie)
        .body(axum::body::Body::from(body)).unwrap();
    let resp = ta.app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 400, "missing url should return 400");
}

#[tokio::test]
async fn test_delete_attachment() {
    let ta = setup().await;
    let cookie = register(&ta.app, "attuser3").await;
    let board_id = create_board(&ta.app, &cookie).await;
    let list_id = create_list(&ta.app, &cookie, &board_id).await;
    let card_id = create_card(&ta.app, &cookie, &list_id).await;

    let att = create_link_attachment(&ta.app, &cookie, &card_id, "To Delete", "https://example.com").await;
    let att_id = att["id"].as_str().unwrap().to_string();

    // Delete it
    let req = axum::http::Request::builder()
        .method("DELETE").uri(&format!("/api/v1/attachments/{att_id}"))
        .header("cookie", &cookie)
        .body(axum::body::Body::empty()).unwrap();
    let resp = ta.app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);

    // Verify it's gone
    let req = axum::http::Request::builder()
        .method("GET").uri(&format!("/api/v1/attachments/card/{card_id}"))
        .header("cookie", &cookie)
        .body(axum::body::Body::empty()).unwrap();
    let resp = ta.app.oneshot(req).await.unwrap();
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let list: Vec<serde_json::Value> = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(list.len(), 0, "attachment should be deleted");
}

#[tokio::test]
async fn test_delete_attachment_requires_auth() {
    let ta = setup().await;

    let req = axum::http::Request::builder()
        .method("DELETE").uri("/api/v1/attachments/00000000-0000-0000-0000-000000000000")
        .body(axum::body::Body::empty()).unwrap();
    let resp = ta.app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 401);
}

#[tokio::test]
async fn test_attachments_scoped_to_card() {
    let ta = setup().await;
    let cookie = register(&ta.app, "attuser4").await;
    let board_id = create_board(&ta.app, &cookie).await;
    let list_id = create_list(&ta.app, &cookie, &board_id).await;
    let card1 = create_card(&ta.app, &cookie, &list_id).await;
    let card2 = create_card(&ta.app, &cookie, &list_id).await;

    create_link_attachment(&ta.app, &cookie, &card1, "Card1 Att", "https://example.com/1").await;

    // Card2 should have no attachments
    let req = axum::http::Request::builder()
        .method("GET").uri(&format!("/api/v1/attachments/card/{card2}"))
        .header("cookie", &cookie)
        .body(axum::body::Body::empty()).unwrap();
    let resp = ta.app.clone().oneshot(req).await.unwrap();
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let list: Vec<serde_json::Value> = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(list.len(), 0, "card2 should have no attachments");
}

#[tokio::test]
async fn test_attachment_service_crud() {
    let ta = setup().await;
    let cookie = register(&ta.app, "attsvc").await;
    let board_id = create_board(&ta.app, &cookie).await;
    let list_id = create_list(&ta.app, &cookie, &board_id).await;
    let card_id = create_card(&ta.app, &cookie, &list_id).await;

    let uid: uuid::Uuid = sqlx::query_scalar("SELECT id FROM users WHERE username = $1")
        .bind("attsvc")
        .fetch_one(&ta._pool)
        .await
        .unwrap();

    let svc = quest_board::services::AttachmentService::new(ta._pool.clone());
    let card_uuid: uuid::Uuid = card_id.parse().unwrap();

    // Create link attachment via service
    let att = svc.create_link(&card_uuid, &uid, "Example", "https://example.com").await.unwrap();
    assert_eq!(att.name, "Example");
    assert_eq!(att.link_url.as_deref(), Some("https://example.com"));

    // List attachments via service
    let list = svc.list_by_card(&card_uuid).await.unwrap();
    assert_eq!(list.len(), 1);

    // Delete via service
    svc.delete(&att.id).await.unwrap();
    let list = svc.list_by_card(&card_uuid).await.unwrap();
    assert!(list.is_empty(), "attachment should be deleted");
}
