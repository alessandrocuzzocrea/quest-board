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
    let state = Arc::new(AppState { db: pool.clone() });
    let app = quest_board::build_app(pool.clone(), state).await;
    (app, pool)
}

async fn register(app: &axum::Router) -> String {
    let req = axum::http::Request::builder()
        .method("POST").uri("/api/v1/auth/register")
        .header("content-type", "application/json")
        .body(axum::body::Body::from(r#"{"email":"del@test.com","password":"pass","name":"Del"}"#)).unwrap();
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
    let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    (v["id"].as_str().unwrap().to_string(), v["slug"].as_str().unwrap().to_string())
}

async fn get_list(app: &axum::Router, cookie: &str, board_id: &str) -> String {
    let req = axum::http::Request::builder()
        .method("GET").uri(&format!("/api/v1/boards/{board_id}"))
        .header("cookie", cookie)
        .body(axum::body::Body::empty()).unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    v["lists"][0]["id"].as_str().unwrap().to_string()
}

async fn create_card(app: &axum::Router, cookie: &str, list_id: &str) -> String {
    let body = format!(r#"{{"list_id":"{list_id}","name":"To Delete"}}"#);
    let req = axum::http::Request::builder()
        .method("POST").uri("/api/v1/cards")
        .header("content-type", "application/json").header("cookie", cookie)
        .body(axum::body::Body::from(body)).unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    v["id"].as_str().unwrap().to_string()
}

async fn create_list(app: &axum::Router, cookie: &str, board_id: &str) -> String {
    let body = format!(r#"{{"board_id":"{board_id}","name":"To Delete"}}"#);
    let req = axum::http::Request::builder()
        .method("POST").uri("/api/v1/lists")
        .header("content-type", "application/json").header("cookie", cookie)
        .body(axum::body::Body::from(body)).unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    v["id"].as_str().unwrap().to_string()
}

#[tokio::test]
async fn test_delete_card_requires_auth() {
    let (app, _pool) = setup().await;
    let req = axum::http::Request::builder()
        .method("DELETE").uri("/api/v1/cards/some-id")
        .body(axum::body::Body::empty()).unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 401);
}

#[tokio::test]
async fn test_delete_card() {
    let (app, _pool) = setup().await;
    let cookie = register(&app).await;
    let (board_id, _) = create_board(&app, &cookie).await;
    let list_id = get_list(&app, &cookie, &board_id).await;
    let card_id = create_card(&app, &cookie, &list_id).await;

    let req = axum::http::Request::builder()
        .method("DELETE").uri(&format!("/api/v1/cards/{card_id}"))
        .header("cookie", &cookie)
        .body(axum::body::Body::empty()).unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200, "delete card");

    // Verify card is gone
    let req = axum::http::Request::builder()
        .method("GET").uri(&format!("/api/v1/cards/{card_id}"))
        .header("cookie", &cookie)
        .body(axum::body::Body::empty()).unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 404, "card should be gone");
}

#[tokio::test]
async fn test_delete_list() {
    let (app, _pool) = setup().await;
    let cookie = register(&app).await;
    let (board_id, _) = create_board(&app, &cookie).await;
    let list_id = create_list(&app, &cookie, &board_id).await;

    let req = axum::http::Request::builder()
        .method("DELETE").uri(&format!("/api/v1/lists/{list_id}"))
        .header("cookie", &cookie)
        .body(axum::body::Body::empty()).unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200, "delete list");

    // Verify list is gone from board
    let req = axum::http::Request::builder()
        .method("GET").uri(&format!("/api/v1/boards/{board_id}"))
        .header("cookie", &cookie)
        .body(axum::body::Body::empty()).unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    let list_ids: Vec<&str> = v["lists"].as_array().unwrap().iter()
        .filter_map(|l| l["id"].as_str()).collect();
    assert!(!list_ids.contains(&list_id.as_str()), "list should be gone");
}

#[tokio::test]
async fn test_delete_list_requires_auth() {
    let (app, _pool) = setup().await;
    let req = axum::http::Request::builder()
        .method("DELETE").uri("/api/v1/lists/some-id")
        .body(axum::body::Body::empty()).unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 401);
}
