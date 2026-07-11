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

async fn register(app: &axum::Router) -> String {
    let req = axum::http::Request::builder()
        .method("POST").uri("/api/v1/auth/register")
        .header("content-type", "application/json")
        .body(axum::body::Body::from(r#"{"email":"edit@test.com","password":"pass","name":"Edit Tester"}"#)).unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);
    resp.headers().get("set-cookie").and_then(|v| v.to_str().ok())
        .map(|s| s.split(';').next().unwrap_or("").to_string()).unwrap()
}

async fn create_board(app: &axum::Router, cookie: &str) -> serde_json::Value {
    let req = axum::http::Request::builder()
        .method("POST").uri("/api/v1/boards")
        .header("content-type", "application/json").header("cookie", cookie)
        .body(axum::body::Body::from(r#"{"name":"Edit Test Board"}"#)).unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

async fn create_list(app: &axum::Router, cookie: &str, board_id: &str) -> serde_json::Value {
    let body = format!(r#"{{"board_id":"{board_id}","name":"To Do"}}"#);
    let req = axum::http::Request::builder()
        .method("POST").uri("/api/v1/lists")
        .header("content-type", "application/json").header("cookie", cookie)
        .body(axum::body::Body::from(body)).unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

async fn create_card(app: &axum::Router, cookie: &str, list_id: &str, name: &str) -> serde_json::Value {
    let body = format!(r#"{{"list_id":"{list_id}","name":"{name}"}}"#);
    let req = axum::http::Request::builder()
        .method("POST").uri("/api/v1/cards")
        .header("content-type", "application/json").header("cookie", cookie)
        .body(axum::body::Body::from(body)).unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

async fn create_checklist(app: &axum::Router, cookie: &str, card_id: &str) -> serde_json::Value {
    let body = format!(r#"{{"card_id":"{card_id}","name":"Steps"}}"#);
    let req = axum::http::Request::builder()
        .method("POST").uri(&format!("/api/v1/cards/{card_id}/task-lists"))
        .header("content-type", "application/json").header("cookie", cookie)
        .body(axum::body::Body::from(body)).unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

async fn create_task(app: &axum::Router, cookie: &str, card_id: &str, tlid: &str, name: &str) -> serde_json::Value {
    let body = format!(r#"{{"name":"{name}"}}"#);
    let req = axum::http::Request::builder()
        .method("POST").uri(&format!("/api/v1/cards/{card_id}/task-lists/{tlid}/tasks"))
        .header("content-type", "application/json").header("cookie", cookie)
        .body(axum::body::Body::from(body)).unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

#[tokio::test]
async fn test_update_card_name() {
    let (app, _pool) = setup().await;
    let cookie = register(&app).await;
    let board = create_board(&app, &cookie).await;
    let list = create_list(&app, &cookie, board["id"].as_str().unwrap()).await;
    let card = create_card(&app, &cookie, list["id"].as_str().unwrap(), "Old Name").await;
    let card_id = card["id"].as_str().unwrap();

    let req = axum::http::Request::builder()
        .method("PUT").uri(&format!("/api/v1/cards/{card_id}"))
        .header("content-type", "application/json").header("cookie", &cookie)
        .body(axum::body::Body::from(r#"{"name":"New Name"}"#)).unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200, "update card name");

    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let updated: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(updated["name"], "New Name");
}

#[tokio::test]
async fn test_update_card_description() {
    let (app, _pool) = setup().await;
    let cookie = register(&app).await;
    let board = create_board(&app, &cookie).await;
    let list = create_list(&app, &cookie, board["id"].as_str().unwrap()).await;
    let card = create_card(&app, &cookie, list["id"].as_str().unwrap(), "Card").await;
    let card_id = card["id"].as_str().unwrap();

    let req = axum::http::Request::builder()
        .method("PUT").uri(&format!("/api/v1/cards/{card_id}"))
        .header("content-type", "application/json").header("cookie", &cookie)
        .body(axum::body::Body::from(r#"{"description":"New description here"}"#)).unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);

    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let updated: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(updated["description"], "New description here");
}

#[tokio::test]
async fn test_update_card_due_date() {
    let (app, _pool) = setup().await;
    let cookie = register(&app).await;
    let board = create_board(&app, &cookie).await;
    let list = create_list(&app, &cookie, board["id"].as_str().unwrap()).await;
    let card = create_card(&app, &cookie, list["id"].as_str().unwrap(), "Card").await;
    let card_id = card["id"].as_str().unwrap();

    let req = axum::http::Request::builder()
        .method("PUT").uri(&format!("/api/v1/cards/{card_id}"))
        .header("content-type", "application/json").header("cookie", &cookie)
        .body(axum::body::Body::from(r#"{"due_date":"2026-08-15T00:00:00Z"}"#)).unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);

    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let updated: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(updated["due_date"], "2026-08-15T00:00:00Z");
}

#[tokio::test]
async fn test_toggle_checklist_task() {
    let (app, _pool) = setup().await;
    let cookie = register(&app).await;
    let board = create_board(&app, &cookie).await;
    let list = create_list(&app, &cookie, board["id"].as_str().unwrap()).await;
    let card = create_card(&app, &cookie, list["id"].as_str().unwrap(), "Card").await;
    let card_id = card["id"].as_str().unwrap();
    let tl = create_checklist(&app, &cookie, card_id).await;
    let tlid = tl["id"].as_str().unwrap();
    let task = create_task(&app, &cookie, card_id, tlid, "Step 1").await;
    let tid = task["id"].as_str().unwrap();

    // Toggle on
    let req = axum::http::Request::builder()
        .method("PUT").uri(&format!("/api/v1/cards/{card_id}/task-lists/{tlid}/tasks/{tid}"))
        .header("content-type", "application/json").header("cookie", &cookie)
        .body(axum::body::Body::from(r#"{"is_completed":true}"#)).unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200, "toggle task on");

    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let updated: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(updated["is_completed"], true);

    // Toggle off
    let req = axum::http::Request::builder()
        .method("PUT").uri(&format!("/api/v1/cards/{card_id}/task-lists/{tlid}/tasks/{tid}"))
        .header("content-type", "application/json").header("cookie", &cookie)
        .body(axum::body::Body::from(r#"{"is_completed":false}"#)).unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200, "toggle task off");

    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let updated: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(updated["is_completed"], false);
}

#[tokio::test]
async fn test_update_card_requires_auth() {
    let (app, _pool) = setup().await;
    let req = axum::http::Request::builder()
        .method("PUT").uri("/api/v1/cards/some-id")
        .header("content-type", "application/json")
        .body(axum::body::Body::from(r#"{"name":"Hack"}"#)).unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 401, "unauthenticated update rejected");
}
