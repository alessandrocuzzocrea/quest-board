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

    // Clean slate: drop and recreate public schema
    sqlx::query("DROP SCHEMA public CASCADE").execute(&pool).await.ok();
    sqlx::query("CREATE SCHEMA public").execute(&pool).await.ok();
    sqlx::query("GRANT ALL ON SCHEMA public TO postgres").execute(&pool).await.ok();
    sqlx::query("GRANT ALL ON SCHEMA public TO public").execute(&pool).await.ok();

    quest_board::db::run_migrations(&pool)
        .await
        .expect("failed to run migrations");

    let state = Arc::new(AppState { db: pool.clone(), ai_client: Arc::new(quest_board::handlers::ai::RealLlmClient) });
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

// ── /index.html should NOT be served ─────────────────────────────────

#[tokio::test]
async fn test_index_html_not_served() {
    let ta = setup().await;

    let req = axum::http::Request::builder()
        .method("GET")
        .uri("/index.html")
        .body(axum::body::Body::empty())
        .unwrap();

    let resp = ta.app.oneshot(req).await.unwrap();

    // index.html should NOT be served — either redirect or 404
    assert!(
        resp.status() != 200,
        "index.html should not be served (got 200)"
    );
}

// ── Protected HTML pages redirect to /login when unauthenticated ─────

#[tokio::test]
async fn test_boards_html_redirects_when_unauthed() {
    let ta = setup().await;

    let req = axum::http::Request::builder()
        .method("GET")
        .uri("/boards.html")
        .body(axum::body::Body::empty())
        .unwrap();

    let resp = ta.app.oneshot(req).await.unwrap();

    // Should redirect to /login
    assert_eq!(resp.status(), 303);
    assert_eq!(
        resp.headers().get("location").and_then(|v| v.to_str().ok()),
        Some("/login")
    );
}

#[tokio::test]
async fn test_board_html_redirects_when_unauthed() {
    let ta = setup().await;

    let req = axum::http::Request::builder()
        .method("GET")
        .uri("/board.html")
        .body(axum::body::Body::empty())
        .unwrap();

    let resp = ta.app.oneshot(req).await.unwrap();

    assert_eq!(resp.status(), 303);
    assert_eq!(
        resp.headers().get("location").and_then(|v| v.to_str().ok()),
        Some("/login")
    );
}

#[tokio::test]
async fn test_settings_html_redirects_when_unauthed() {
    let ta = setup().await;

    let req = axum::http::Request::builder()
        .method("GET")
        .uri("/settings.html")
        .body(axum::body::Body::empty())
        .unwrap();

    let resp = ta.app.oneshot(req).await.unwrap();

    assert_eq!(resp.status(), 303);
    assert_eq!(
        resp.headers().get("location").and_then(|v| v.to_str().ok()),
        Some("/login")
    );
}

// ── Protected HTML pages serve when authenticated ────────────────────

#[tokio::test]
async fn test_boards_html_serves_when_authed() {
    let ta = setup().await;
    let cookie = register(&ta.app).await;

    let req = axum::http::Request::builder()
        .method("GET")
        .uri("/boards.html")
        .header("cookie", &cookie)
        .body(axum::body::Body::empty())
        .unwrap();

    let resp = ta.app.oneshot(req).await.unwrap();

    assert_eq!(
        resp.status(),
        200,
        "authenticated user should be able to access boards.html"
    );
}

// ── Login page is always accessible ──────────────────────────────────

#[tokio::test]
async fn test_login_page_always_accessible() {
    let ta = setup().await;

    let req = axum::http::Request::builder()
        .method("GET")
        .uri("/login")
        .body(axum::body::Body::empty())
        .unwrap();

    let resp = ta.app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);
}

// ── Static assets (CSS, JS) are always accessible ────────────────────

#[tokio::test]
async fn test_css_always_accessible() {
    let ta = setup().await;

    let req = axum::http::Request::builder()
        .method("GET")
        .uri("/css/style.css")
        .body(axum::body::Body::empty())
        .unwrap();

    let resp = ta.app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);
}

// ── Clean URLs (without .html) redirect when unauthenticated ────────

#[tokio::test]
async fn test_boards_clean_url_redirects_when_unauthed() {
    let ta = setup().await;

    let req = axum::http::Request::builder()
        .method("GET")
        .uri("/boards")
        .body(axum::body::Body::empty())
        .unwrap();

    let resp = ta.app.oneshot(req).await.unwrap();

    assert_eq!(resp.status(), 303);
    assert_eq!(
        resp.headers().get("location").and_then(|v| v.to_str().ok()),
        Some("/login")
    );
}

#[tokio::test]
async fn test_board_clean_url_redirects_when_unauthed() {
    let ta = setup().await;

    let req = axum::http::Request::builder()
        .method("GET")
        .uri("/board")
        .body(axum::body::Body::empty())
        .unwrap();

    let resp = ta.app.oneshot(req).await.unwrap();

    assert_eq!(resp.status(), 303);
    assert_eq!(
        resp.headers().get("location").and_then(|v| v.to_str().ok()),
        Some("/login")
    );
}

#[tokio::test]
async fn test_settings_clean_url_redirects_when_unauthed() {
    let ta = setup().await;

    let req = axum::http::Request::builder()
        .method("GET")
        .uri("/settings")
        .body(axum::body::Body::empty())
        .unwrap();

    let resp = ta.app.oneshot(req).await.unwrap();

    assert_eq!(resp.status(), 303);
    assert_eq!(
        resp.headers().get("location").and_then(|v| v.to_str().ok()),
        Some("/login")
    );
}

#[tokio::test]
async fn test_boards_clean_url_serves_when_authed() {
    let ta = setup().await;
    let cookie = register(&ta.app).await;

    let req = axum::http::Request::builder()
        .method("GET")
        .uri("/boards")
        .header("cookie", &cookie)
        .body(axum::body::Body::empty())
        .unwrap();

    let resp = ta.app.oneshot(req).await.unwrap();

    assert_eq!(
        resp.status(),
        200,
        "authenticated user should be able to access /boards"
    );
}

// ── Root path / redirects based on auth status ──────────────────────

#[tokio::test]
async fn test_root_redirects_to_login_when_unauthed() {
    let ta = setup().await;

    let req = axum::http::Request::builder()
        .method("GET")
        .uri("/")
        .body(axum::body::Body::empty())
        .unwrap();

    let resp = ta.app.oneshot(req).await.unwrap();

    assert_eq!(resp.status(), 303);
    assert_eq!(
        resp.headers().get("location").and_then(|v| v.to_str().ok()),
        Some("/login")
    );
}

#[tokio::test]
async fn test_root_redirects_to_boards_when_authed() {
    let ta = setup().await;
    let cookie = register(&ta.app).await;

    let req = axum::http::Request::builder()
        .method("GET")
        .uri("/")
        .header("cookie", &cookie)
        .body(axum::body::Body::empty())
        .unwrap();

    let resp = ta.app.oneshot(req).await.unwrap();

    assert_eq!(resp.status(), 303);
    assert_eq!(
        resp.headers().get("location").and_then(|v| v.to_str().ok()),
        Some("/boards")
    );
}
