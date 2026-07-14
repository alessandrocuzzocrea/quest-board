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

async fn register(app: &axum::Router) -> String {
    let req = axum::http::Request::builder()
        .method("POST").uri("/api/v1/auth/register")
        .header("content-type", "application/json")
        .body(axum::body::Body::from(r#"{"username":"edit","password":"pass","name":"Edit Tester"}"#)).unwrap();
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
    let body = format!(r#"{{"board_id":"{board_id}","name":"Test List"}}"#);
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

#[tokio::test]
async fn test_rename_card() {
    let ta = setup().await;
    let cookie = register(&ta.app).await;
    let board = create_board(&ta.app, &cookie).await;
    let list = create_list(&ta.app, &cookie, board["id"].as_str().unwrap()).await;
    let card = create_card(&ta.app, &cookie, list["id"].as_str().unwrap(), "Old Name").await;
    let card_id = card["id"].as_str().unwrap();

    let req = axum::http::Request::builder()
        .method("PUT").uri(format!("/api/v1/cards/{card_id}"))
        .header("content-type", "application/json").header("cookie", &cookie)
        .body(axum::body::Body::from(r#"{"name":"New Name"}"#)).unwrap();
    let resp = ta.app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let card: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(card["name"], "New Name");
}

#[tokio::test]
async fn test_move_card() {
    let ta = setup().await;
    let cookie = register(&ta.app).await;
    let board = create_board(&ta.app, &cookie).await;
    let list1 = create_list(&ta.app, &cookie, board["id"].as_str().unwrap()).await;
    let list2 = create_list(&ta.app, &cookie, board["id"].as_str().unwrap()).await;
    let card = create_card(&ta.app, &cookie, list1["id"].as_str().unwrap(), "Card").await;
    let card_id = card["id"].as_str().unwrap();

    let body = format!(r#"{{"list_id":"{}","position":1.0}}"#, list2["id"].as_str().unwrap());
    let req = axum::http::Request::builder()
        .method("PUT").uri(format!("/api/v1/cards/{card_id}/move"))
        .header("content-type", "application/json").header("cookie", &cookie)
        .body(axum::body::Body::from(body)).unwrap();
    let resp = ta.app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let card: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(card["list_id"], list2["id"]);
}

#[tokio::test]
async fn test_add_and_remove_card_member() {
    let ta = setup().await;
    let cookie = register(&ta.app).await;
    let board = create_board(&ta.app, &cookie).await;
    let list = create_list(&ta.app, &cookie, board["id"].as_str().unwrap()).await;
    let card = create_card(&ta.app, &cookie, list["id"].as_str().unwrap(), "Card").await;
    let card_id = card["id"].as_str().unwrap();

    // Get the logged-in user's ID
    let req = axum::http::Request::builder()
        .method("GET").uri("/api/v1/auth/me")
        .header("cookie", &cookie)
        .body(axum::body::Body::empty()).unwrap();
    let resp = ta.app.clone().oneshot(req).await.unwrap();
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let me: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    let user_id = me["user"]["id"].as_str().unwrap();

    // Add member
    let body = format!(r#"{{"user_id":"{user_id}"}}"#);
    let resp = ta.app.clone().oneshot(
        axum::http::Request::builder()
            .method("POST").uri(format!("/api/v1/cards/{card_id}/members"))
            .header("content-type", "application/json").header("cookie", &cookie)
            .body(axum::body::Body::from(body)).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), 200);

    // Remove member
    let body = format!(r#"{{"user_id":"{user_id}"}}"#);
    let resp = ta.app.clone().oneshot(
        axum::http::Request::builder()
            .method("DELETE").uri(format!("/api/v1/cards/{card_id}/members"))
            .header("content-type", "application/json").header("cookie", &cookie)
            .body(axum::body::Body::from(body)).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), 200);
}

async fn create_label(app: &axum::Router, cookie: &str, board_id: &str, name: &str) -> serde_json::Value {
    let color = "#0079bf";
    let body = format!(r#"{{"board_id":"{board_id}","name":"{name}","color":"{color}"}}"#);
    let req = axum::http::Request::builder()
        .method("POST").uri("/api/v1/labels")
        .header("content-type", "application/json").header("cookie", cookie)
        .body(axum::body::Body::from(body)).unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

#[tokio::test]
async fn test_add_and_remove_card_label() {
    let ta = setup().await;
    let cookie = register(&ta.app).await;
    let board = create_board(&ta.app, &cookie).await;
    let board_id = board["id"].as_str().unwrap();
    let label = create_label(&ta.app, &cookie, board_id, "Bug").await;
    let label_id = label["id"].as_str().unwrap();
    let list = create_list(&ta.app, &cookie, board_id).await;
    let card = create_card(&ta.app, &cookie, list["id"].as_str().unwrap(), "Card").await;
    let card_id = card["id"].as_str().unwrap();

    // Add label
    let body = format!(r#"{{"label_id":"{label_id}"}}"#);
    let resp = ta.app.clone().oneshot(
        axum::http::Request::builder()
            .method("POST").uri(format!("/api/v1/cards/{card_id}/labels"))
            .header("content-type", "application/json").header("cookie", &cookie)
            .body(axum::body::Body::from(body)).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), 200);

    // Remove label
    let body = format!(r#"{{"label_id":"{label_id}"}}"#);
    let resp = ta.app.clone().oneshot(
        axum::http::Request::builder()
            .method("DELETE").uri(format!("/api/v1/cards/{card_id}/labels"))
            .header("content-type", "application/json").header("cookie", &cookie)
            .body(axum::body::Body::from(body)).unwrap()
    ).await.unwrap();
    assert_eq!(resp.status(), 200);
}
