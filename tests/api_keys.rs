use quest_board::AppState;
use std::sync::Arc;
use std::sync::LazyLock;
use tokio::sync::Mutex;
use tower::ServiceExt;

static SETUP_MUTEX: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

async fn setup() -> (axum::Router, sqlx::PgPool) {
    let _guard = SETUP_MUTEX.lock().await;
    dotenvy::from_filename(".env.test").ok();

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:quest@localhost:5432/quest_test".into());

    let pool = sqlx::PgPool::connect(&database_url)
        .await
        .expect("failed to connect");

    sqlx::query("DROP TABLE IF EXISTS api_keys, sessions, favorites, notifications, actions, tasks, task_lists, attachments, comments, card_labels, labels, card_members, cards, lists, board_members, boards, users CASCADE")
        .execute(&pool).await.ok();

    quest_board::db::run_migrations(&pool)
        .await
        .expect("failed to run migrations");

    let state = Arc::new(AppState { db: pool.clone(), ai_client: Arc::new(quest_board::handlers::ai::RealLlmClient) });
    let app = quest_board::build_app(pool.clone(), state).await;
    (app, pool)
}

async fn register(app: &axum::Router, email: &str) -> (axum::Router, String) {
    let body = format!(r#"{{"email":"{email}","password":"pass","name":"T"}}"#);
    let req = axum::http::Request::builder()
        .method("POST").uri("/api/v1/auth/register")
        .header("content-type", "application/json")
        .body(axum::body::Body::from(body)).unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200, "register {email}");
    let cookie = resp.headers().get("set-cookie")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.split(';').next().unwrap_or("").to_string())
        .unwrap();
    (app.clone(), cookie)
}

fn json_request(method: &str, uri: &str, body: &str, cookie: Option<&str>) -> axum::http::Request<axum::body::Body> {
    let mut builder = axum::http::Request::builder()
        .method(method).uri(uri)
        .header("content-type", "application/json");
    if let Some(c) = cookie {
        builder = builder.header("cookie", c);
    }
    builder.body(axum::body::Body::from(body.to_string())).unwrap()
}

fn empty_request(method: &str, uri: &str, cookie: Option<&str>) -> axum::http::Request<axum::body::Body> {
    let mut builder = axum::http::Request::builder()
        .method(method).uri(uri);
    if let Some(c) = cookie {
        builder = builder.header("cookie", c);
    }
    builder.body(axum::body::Body::empty()).unwrap()
}

fn bearer_request(method: &str, uri: &str, token: &str) -> axum::http::Request<axum::body::Body> {
    axum::http::Request::builder()
        .method(method).uri(uri)
        .header("authorization", format!("Bearer {token}"))
        .body(axum::body::Body::empty()).unwrap()
}

async fn body(resp: axum::http::Response<axum::body::Body>) -> serde_json::Value {
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&bytes).unwrap_or(serde_json::Value::Null)
}

#[tokio::test]
async fn test_create_api_key_requires_auth() {
    let (app, _pool) = setup().await;
    let resp = app.oneshot(json_request("POST", "/api/v1/api-keys", r#"{"name":"test"}"#, None)).await.unwrap();
    assert_eq!(resp.status(), 401, "unauthenticated should be 401");
}

#[tokio::test]
async fn test_create_and_use_api_key() {
    let (app, cookie) = register(&setup().await.0, "key-test@test.com").await;

    // Create key
    let resp = app.clone().oneshot(json_request("POST", "/api/v1/api-keys", r#"{"name":"my-token"}"#, Some(&cookie))).await.unwrap();
    assert_eq!(resp.status(), 200, "create key");
    let json = body(resp).await;
    let token = json["token"].as_str().expect("token in response").to_string();
    assert!(token.starts_with("qb_"), "token starts with qb_: {token}");
    assert_eq!(token.len(), 46, "qb_ + 43 base64url chars = 46");
    assert_eq!(json["api_key"]["name"], "my-token");
    assert!(json["api_key"]["prefix"].as_str().unwrap().len() >= 6, "has prefix");

    // List with cookie
    let resp = app.clone().oneshot(empty_request("GET", "/api/v1/api-keys", Some(&cookie))).await.unwrap();
    assert_eq!(resp.status(), 200);
    let list = body(resp).await;
    assert!(list.as_array().unwrap().len() >= 1);

    // List with cookie
    let resp = app.clone().oneshot(empty_request("GET", "/api/v1/api-keys", Some(&cookie))).await.unwrap();
    assert_eq!(resp.status(), 200);
    let list = body(resp).await;
    assert!(list.as_array().unwrap().len() >= 1, "should have keys");
    let key_id = list[0]["id"].as_str().unwrap().to_string();

    // List with bearer token
    let resp = app.clone().oneshot(bearer_request("GET", "/api/v1/api-keys", &token)).await.unwrap();
    assert_eq!(resp.status(), 200, "bearer auth should work");
    let list2 = body(resp).await;
    assert!(list2.as_array().unwrap().len() >= 1, "bearer should see keys");
    assert_eq!(list2[0]["id"], key_id, "same key returned via bearer");

    // Access boards with bearer token
    let resp = app.clone().oneshot(bearer_request("GET", "/api/v1/boards", &token)).await.unwrap();
    assert_eq!(resp.status(), 200, "bearer token should work on boards endpoint");
}

#[tokio::test]
async fn test_invalid_bearer_token_rejected() {
    let (app, _pool) = setup().await;
    let resp = app.oneshot(bearer_request("GET", "/api/v1/api-keys", "qb_invalidtoken123")).await.unwrap();
    assert_eq!(resp.status(), 401, "invalid token rejected");
}

#[tokio::test]
async fn test_delete_api_key() {
    let (app, cookie) = register(&setup().await.0, "delete-key@test.com").await;

    // Create
    let resp = app.clone().oneshot(json_request("POST", "/api/v1/api-keys", r#"{"name":"delete-me"}"#, Some(&cookie))).await.unwrap();
    let json = body(resp).await;
    let key_id = json["api_key"]["id"].as_str().unwrap().to_string();
    let token = json["token"].as_str().unwrap().to_string();

    // Delete
    let resp = app.clone().oneshot(json_request("DELETE", &format!("/api/v1/api-keys/{key_id}"), "", Some(&cookie))).await.unwrap();
    assert_eq!(resp.status(), 200, "delete should succeed");

    // Token should no longer work
    let resp = app.clone().oneshot(bearer_request("GET", "/api/v1/boards", &token)).await.unwrap();
    assert_eq!(resp.status(), 401, "deleted token rejected");

    // List should not show deleted key
    let resp = app.clone().oneshot(empty_request("GET", "/api/v1/api-keys", Some(&cookie))).await.unwrap();
    let list = body(resp).await;
    let ids: Vec<&str> = list.as_array().unwrap().iter().filter_map(|k| k["id"].as_str()).collect();
    assert!(!ids.contains(&key_id.as_str()), "deleted key not in list");
}

#[tokio::test]
async fn test_multiple_api_keys_per_user() {
    let (app, cookie) = register(&setup().await.0, "multi-key@test.com").await;

    for name in ["ci", "dev", "prod"] {
        let resp = app.clone().oneshot(json_request("POST", "/api/v1/api-keys", &format!(r#"{{"name":"{name}"}}"#), Some(&cookie))).await.unwrap();
        assert_eq!(resp.status(), 200, "create {name}");
    }

    let resp = app.clone().oneshot(empty_request("GET", "/api/v1/api-keys", Some(&cookie))).await.unwrap();
    let list = body(resp).await;
    assert_eq!(list.as_array().unwrap().len(), 3, "should have 3 keys");
}
