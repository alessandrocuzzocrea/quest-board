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
    let state = Arc::new(AppState { db: pool.clone(), event_tx });
    let app = quest_board::build_app(pool.clone(), state).await;

    TestApp { _guard: guard, app, _pool: pool }
}

async fn register(app: &axum::Router) -> String {
    let req = axum::http::Request::builder()
        .method("POST")
        .uri("/api/v1/auth/register")
        .header("content-type", "application/json")
        .body(axum::body::Body::from(
            r#"{"username":"testuser","password":"secret"}"#,
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
            r#"{"username":"testuser","password":"secret"}"#,
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
            r#"{"username":"me","password":"pass"}"#,
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
async fn test_change_password() {
    let ta = setup().await;
    let cookie = register(&ta.app).await;

    // Change password
    let req = axum::http::Request::builder()
        .method("PUT").uri("/api/v1/auth/me/password")
        .header("content-type", "application/json")
        .header("cookie", &cookie)
        .body(axum::body::Body::from(r#"{"current_password":"secret","new_password":"new-secret"}"#)).unwrap();
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
        .body(axum::body::Body::from(r#"{"current_password":"wrong","new_password":"new-secret"}"#)).unwrap();
    let resp = ta.app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 401);
}

#[tokio::test]
async fn test_change_password_requires_auth() {
    let ta = setup().await;
    let req = axum::http::Request::builder()
        .method("PUT").uri("/api/v1/auth/me/password")
        .header("content-type", "application/json")
        .body(axum::body::Body::from(r#"{"current_password":"x","new_password":"y"}"#)).unwrap();
    let resp = ta.app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 401);
}


#[tokio::test]
async fn test_auth_service_register_login() {
    let ta = setup().await;
    let svc = quest_board::services::AuthService::new(ta._pool.clone());

    // Register via service
    let user = svc.register("svcuser", "secret").await.unwrap();
    assert_eq!(user.username, "svcuser");

    // Login via service — password hash should match
    let logged_in = svc.login("svcuser", "secret").await.unwrap();
    assert_eq!(logged_in.id, user.id);

    // Wrong password
    let err = svc.login("svcuser", "wrong").await.unwrap_err();
    assert!(matches!(err, quest_board::error::AppError::Unauthorized(_)));

    // Get user via service
    let fetched = svc.get_user(&user.id).await.unwrap();
    assert_eq!(fetched.username, "svcuser");

    // Change password via service
    svc.change_password(&user.id, "secret", "newsecret").await.unwrap();
    let err = svc.login("svcuser", "secret").await.unwrap_err();
    assert!(matches!(err, quest_board::error::AppError::Unauthorized(_)));
    let logged_in = svc.login("svcuser", "newsecret").await.unwrap();
    assert_eq!(logged_in.id, user.id);

}
