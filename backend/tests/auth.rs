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

    let pool = sqlx::PgPool::connect(&database_url)
        .await
        .expect("failed to connect");

    // Clean slate: drop all tables, re-run migrations
    sqlx::query(
        "DROP TABLE IF EXISTS sessions, favorites, notifications, actions, tasks, task_lists, \
         attachments, comments, card_labels, labels, card_members, cards, lists, \
         board_members, boards, users CASCADE",
    )
    .execute(&pool)
    .await
    .ok();

    quest_board::db::run_migrations(&pool)
        .await
        .expect("failed to run migrations");

    let state = Arc::new(AppState { db: pool.clone() });
    let app = quest_board::build_app(pool.clone(), state).await;

    (app, pool)
}

#[tokio::test]
async fn test_register_and_login() {
    let (app, _pool) = setup().await;

    let req = axum::http::Request::builder()
        .method("POST")
        .uri("/api/v1/auth/register")
        .header("content-type", "application/json")
        .body(axum::body::Body::from(
            r#"{"email":"test@test.com","password":"secret","name":"Test User"}"#,
        ))
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = serde_json::from_slice(
        &axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap(),
    )
    .unwrap();
    assert_eq!(body["user"]["email"], "test@test.com");
    assert_eq!(body["user"]["name"], "Test User");
}

#[tokio::test]
async fn test_login_with_wrong_password() {
    let (app, _pool) = setup().await;

    let req = axum::http::Request::builder()
        .method("POST")
        .uri("/api/v1/auth/login")
        .header("content-type", "application/json")
        .body(axum::body::Body::from(
            r#"{"email":"nobody@test.com","password":"wrong"}"#,
        ))
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 401);
}

#[tokio::test]
async fn test_register_then_me() {
    let (app, _pool) = setup().await;

    let req = axum::http::Request::builder()
        .method("POST")
        .uri("/api/v1/auth/register")
        .header("content-type", "application/json")
        .body(axum::body::Body::from(
            r#"{"email":"me@test.com","password":"pass","name":"Me"}"#,
        ))
        .unwrap();

    let resp = app.clone().oneshot(req).await.unwrap();
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

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = serde_json::from_slice(
        &axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap(),
    )
    .unwrap();
    assert_eq!(body["user"]["email"], "me@test.com");
}

#[tokio::test]
async fn test_seeded_admin_login() {
    let (app, _pool) = setup().await;

    let req = axum::http::Request::builder()
        .method("POST")
        .uri("/api/v1/auth/login")
        .header("content-type", "application/json")
        .body(axum::body::Body::from(
            r#"{"email":"admin","password":"admin"}"#,
        ))
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = serde_json::from_slice(
        &axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap(),
    )
    .unwrap();
    assert_eq!(body["user"]["email"], "admin");
    assert_eq!(body["user"]["role"], "admin");
}
