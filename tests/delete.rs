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
    let (event_tx, _) = quest_board::events::channel();
    let state = Arc::new(AppState { db: pool.clone(), ai_client: Arc::new(quest_board::handlers::ai::RealLlmClient), event_tx });
    let app = quest_board::build_app(pool.clone(), state).await;
    TestApp { _guard, app, _pool: pool }
}

async fn register(app: &axum::Router) -> String {
    let req = axum::http::Request::builder()
        .method("POST").uri("/api/v1/auth/register")
        .header("content-type", "application/json")
        .body(axum::body::Body::from(r#"{"username":"del","password":"pass","name":"Del"}"#)).unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);
    resp.headers().get("set-cookie").and_then(|v| v.to_str().ok())
        .map(|s| s.split(';').next().unwrap_or("").to_string()).unwrap()
}

async fn create_board(app: &axum::Router, cookie: &str) -> (String, String) {
    let req = axum::http::Request::builder()
        .method("POST").uri("/api/v1/boards")
        .header("content-type", "application/json").header("cookie", cookie)
        .body(axum::body::Body::from(r#"{"name":"Del Board"}"#)).unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let board: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    (board["id"].as_str().unwrap().to_string(), cookie.to_string())
}

async fn create_list(app: &axum::Router, cookie: &str, board_id: &str) -> String {
    let req = axum::http::Request::builder()
        .method("POST").uri("/api/v1/lists")
        .header("content-type", "application/json").header("cookie", cookie)
        .body(axum::body::Body::from(
            format!(r#"{{"board_id":"{board_id}","name":"List"}}"#)
        )).unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let list: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    list["id"].as_str().unwrap().to_string()
}

async fn create_card(app: &axum::Router, cookie: &str, list_id: &str) -> String {
    let req = axum::http::Request::builder()
        .method("POST").uri("/api/v1/cards")
        .header("content-type", "application/json").header("cookie", cookie)
        .body(axum::body::Body::from(
            format!(r#"{{"list_id":"{list_id}","name":"Card"}}"#)
        )).unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let card: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    card["id"].as_str().unwrap().to_string()
}

fn cookie_request(method: &str, uri: &str, cookie: &str) -> axum::http::Request<axum::body::Body> {
    axum::http::Request::builder()
        .method(method).uri(uri)
        .header("cookie", cookie)
        .body(axum::body::Body::empty()).unwrap()
}

fn json_request(method: &str, uri: &str, body: &str) -> axum::http::Request<axum::body::Body> {
    axum::http::Request::builder()
        .method(method).uri(uri)
        .header("content-type", "application/json")
        .body(axum::body::Body::from(body.to_string())).unwrap()
}

fn json_cookie_request(method: &str, uri: &str, body: &str, cookie: &str) -> axum::http::Request<axum::body::Body> {
    axum::http::Request::builder()
        .method(method).uri(uri)
        .header("content-type", "application/json")
        .header("cookie", cookie)
        .body(axum::body::Body::from(body.to_string())).unwrap()
}

#[tokio::test]
async fn test_unauthorized_requests() {
    let ta = setup().await;

    // All these should return 401
    for (method, uri, body) in [
        ("POST", "/api/v1/boards", r#"{"name":"X"}"#),
        ("DELETE", "/api/v1/boards/00000000-0000-0000-0000-000000000000", ""),
        ("POST", "/api/v1/lists", r#"{"board_id":"00000000-0000-0000-0000-000000000000","name":"X"}"#),
        ("DELETE", "/api/v1/lists/00000000-0000-0000-0000-000000000000", ""),
        ("POST", "/api/v1/cards", r#"{"list_id":"00000000-0000-0000-0000-000000000000","name":"X"}"#),
        ("DELETE", "/api/v1/cards/00000000-0000-0000-0000-000000000000", ""),
        ("DELETE", "/api/v1/comments/00000000-0000-0000-0000-000000000000", ""),
    ] {
        let req = json_request(method, uri, body);
        let resp = ta.app.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), 401, "{method} {uri} should be 401, got {}", resp.status());
    }
}

#[tokio::test]
async fn test_delete_board() {
    let ta = setup().await;
    let cookie = register(&ta.app).await;
    let (board_id, _) = create_board(&ta.app, &cookie).await;
    let list_id = create_list(&ta.app, &cookie, &board_id).await;
    let card_id = create_card(&ta.app, &cookie, &list_id).await;

    // Delete card
    let resp = ta.app.clone().oneshot(cookie_request("DELETE", &format!("/api/v1/cards/{card_id}"), &cookie)).await.unwrap();
    assert_eq!(resp.status(), 200, "delete card");

    // Delete list
    let resp = ta.app.clone().oneshot(cookie_request("DELETE", &format!("/api/v1/lists/{list_id}"), &cookie)).await.unwrap();
    assert_eq!(resp.status(), 200, "delete list");

    // Delete board
    let resp = ta.app.clone().oneshot(cookie_request("DELETE", &format!("/api/v1/boards/{board_id}"), &cookie)).await.unwrap();
    assert_eq!(resp.status(), 200, "delete board");
}

#[tokio::test]
async fn test_delete_comment() {
    let ta = setup().await;
    let cookie = register(&ta.app).await;
    let (board_id, _) = create_board(&ta.app, &cookie).await;
    let list_id = create_list(&ta.app, &cookie, &board_id).await;
    let card_id = create_card(&ta.app, &cookie, &list_id).await;

    // Create comment
    let body = json_cookie_request("POST", "/api/v1/comments", &format!(r#"{{"card_id":"{card_id}","text":"hi"}}"#), &cookie);
    let resp = ta.app.clone().oneshot(body).await.unwrap();
    assert_eq!(resp.status(), 200, "create comment");
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let comment: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    let comment_id = comment["id"].as_str().unwrap();

    // Delete comment
    let resp = ta.app.clone().oneshot(cookie_request("DELETE", &format!("/api/v1/comments/{comment_id}"), &cookie)).await.unwrap();
    assert_eq!(resp.status(), 200, "delete comment");
}
