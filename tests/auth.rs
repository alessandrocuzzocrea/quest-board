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

    // Clean slate: drop all tables, re-run migrations
    sqlx::query(
        "DROP TABLE IF EXISTS api_keys, sessions, favorites, notifications, actions, tasks, task_lists, \
         attachments, comments, card_labels, labels, card_members, cards, lists, \
         board_members, boards, users CASCADE",
    )
    .execute(&pool)
    .await
    .ok();

    quest_board::db::run_migrations(&pool)
        .await
        .expect("failed to run migrations");

    let (event_tx, _) = quest_board::events::channel();
    let state = Arc::new(AppState { db: pool.clone(), ai_client: Arc::new(quest_board::handlers::ai::RealLlmClient), event_tx });
    let app = quest_board::build_app(pool.clone(), state).await;

    TestApp { _guard: guard, app, _pool: pool }
}

async fn register(app: &axum::Router) -> String {
    let req = axum::http::Request::builder()
        .method("POST")
        .uri("/api/v1/auth/register")
        .header("content-type", "application/json")
        .body(axum::body::Body::from(
            r#"{"username":"testuser","password":"secret","name":"Test User"}"#,
        ))
        .unwrap();

    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);

    resp
        .headers()
        .get("set-cookie")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.split(';').next().unwrap_or("").to_string())
        .unwrap_or_default()
}

#[tokio::test]
async fn test_register_and_login() {
    let ta = setup().await;

    let req = axum::http::Request::builder()
        .method("POST")
        .uri("/api/v1/auth/register")
        .header("content-type", "application/json")
        .body(axum::body::Body::from(
            r#"{"username":"testuser","password":"secret","name":"Test User"}"#,
        ))
        .unwrap();

    let resp = ta.app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = serde_json::from_slice(
        &axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap(),
    )
    .unwrap();
    assert_eq!(body["user"]["username"], "testuser");
    assert_eq!(body["user"]["name"], "Test User");
}

#[tokio::test]
async fn test_login_with_wrong_password() {
    let ta = setup().await;

    let req = axum::http::Request::builder()
        .method("POST")
        .uri("/api/v1/auth/login")
        .header("content-type", "application/json")
        .body(axum::body::Body::from(
            r#"{"username":"nobody","password":"wrong"}"#,
        ))
        .unwrap();

    let resp = ta.app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 401);
}

#[tokio::test]
async fn test_register_then_me() {
    let ta = setup().await;

    let req = axum::http::Request::builder()
        .method("POST")
        .uri("/api/v1/auth/register")
        .header("content-type", "application/json")
        .body(axum::body::Body::from(
            r#"{"username":"me","password":"pass","name":"Me"}"#,
        ))
        .unwrap();

    let resp = ta.app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);

    let cookie = resp
        .headers()
        .get("set-cookie")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.split(';').next().unwrap_or("").to_string())
        .unwrap_or_default();

    let req = axum::http::Request::builder()
        .method("GET")
        .uri("/api/v1/auth/me")
        .header("cookie", &cookie)
        .body(axum::body::Body::empty())
        .unwrap();

    let resp = ta.app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = serde_json::from_slice(
        &axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap(),
    )
    .unwrap();
    assert_eq!(body["user"]["username"], "me");
}

#[tokio::test]
async fn test_seeded_admin_login() {
    let ta = setup().await;

    let req = axum::http::Request::builder()
        .method("POST")
        .uri("/api/v1/auth/login")
        .header("content-type", "application/json")
        .body(axum::body::Body::from(
            r#"{"username":"admin","password":"admin"}"#,
        ))
        .unwrap();

    let resp = ta.app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = serde_json::from_slice(
        &axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap(),
    )
    .unwrap();
    assert_eq!(body["user"]["username"], "admin");
    assert_eq!(body["user"]["role"], "admin");
}

// ── User Settings Tests ──────────────────────────────────────────

#[tokio::test]
async fn test_update_name_requires_auth() {
    let ta = setup().await;
    let req = axum::http::Request::builder()
        .method("PUT").uri("/api/v1/auth/me")
        .header("content-type", "application/json")
        .body(axum::body::Body::from(r#"{"name":"New"}"#)).unwrap();
    let resp = ta.app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 401);
}

#[tokio::test]
async fn test_update_name() {
    let ta = setup().await;
    let cookie = register(&ta.app).await;

    // Update name
    let req = axum::http::Request::builder()
        .method("PUT").uri("/api/v1/auth/me")
        .header("content-type", "application/json")
        .header("cookie", &cookie)
        .body(axum::body::Body::from(r#"{"name":"NewName"}"#)).unwrap();
    let resp = ta.app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = serde_json::from_slice(
        &axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap()
    ).unwrap();
    assert_eq!(body["user"]["name"], "NewName");

    // Verify via me endpoint
    let req = axum::http::Request::builder()
        .method("GET").uri("/api/v1/auth/me")
        .header("cookie", &cookie)
        .body(axum::body::Body::empty()).unwrap();
    let resp = ta.app.clone().oneshot(req).await.unwrap();
    let body: serde_json::Value = serde_json::from_slice(
        &axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap()
    ).unwrap();
    assert_eq!(body["user"]["name"], "NewName");
}

#[tokio::test]
async fn test_change_password() {
    let ta = setup().await;
    let cookie = register(&ta.app).await;

    // Change password
    let req = axum::http::Request::builder()
        .method("PUT").uri("/api/v1/auth/me/password")
        .header("content-type", "application/json")
        .header("cookie", &cookie)
        .body(axum::body::Body::from(r#"{"old_password":"secret","new_password":"new-secret"}"#)).unwrap();
    let resp = ta.app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);

    // Verify login with new password
    let req = axum::http::Request::builder()
        .method("POST").uri("/api/v1/auth/login")
        .header("content-type", "application/json")
        .body(axum::body::Body::from(
            r#"{"username":"testuser","password":"new-secret"}"#,
        )).unwrap();
    let resp = ta.app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);

    // Old password should fail
    let req = axum::http::Request::builder()
        .method("POST").uri("/api/v1/auth/login")
        .header("content-type", "application/json")
        .body(axum::body::Body::from(
            r#"{"username":"testuser","password":"secret"}"#,
        )).unwrap();
    let resp = ta.app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 401);
}

#[tokio::test]
async fn test_change_password_wrong_old_password() {
    let ta = setup().await;
    let cookie = register(&ta.app).await;

    let req = axum::http::Request::builder()
        .method("PUT").uri("/api/v1/auth/me/password")
        .header("content-type", "application/json")
        .header("cookie", &cookie)
        .body(axum::body::Body::from(r#"{"old_password":"wrong","new_password":"new-secret"}"#)).unwrap();
    let resp = ta.app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 401);
}

#[tokio::test]
async fn test_change_password_requires_auth() {
    let ta = setup().await;
    let req = axum::http::Request::builder()
        .method("PUT").uri("/api/v1/auth/me/password")
        .header("content-type", "application/json")
        .body(axum::body::Body::from(r#"{"old_password":"x","new_password":"y"}"#)).unwrap();
    let resp = ta.app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 401);
}
