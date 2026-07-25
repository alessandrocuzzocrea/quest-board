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

// ── SPA serves on all frontend routes ─────────────────────────────────

#[tokio::test]
async fn test_spa_serves_on_root() {
    let ta = setup().await;
    let req = axum::http::Request::builder()
        .method("GET").uri("/")
        .body(axum::body::Body::empty()).unwrap();
    let resp = ta.app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);
    let body = axum::body::to_bytes(resp.into_body(), 1024 * 1024).await.unwrap();
    let html = String::from_utf8(body.to_vec()).unwrap();
    assert!(html.contains("root"), "SPA must contain root div");
}

#[tokio::test]
async fn test_spa_serves_on_login() {
    let ta = setup().await;
    let req = axum::http::Request::builder()
        .method("GET").uri("/login")
        .body(axum::body::Body::empty()).unwrap();
    let resp = ta.app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);
    let body = axum::body::to_bytes(resp.into_body(), 1024 * 1024).await.unwrap();
    let html = String::from_utf8(body.to_vec()).unwrap();
    assert!(html.contains("root"), "SPA must contain root div");
}

#[tokio::test]
async fn test_spa_serves_on_boards() {
    let ta = setup().await;
    let req = axum::http::Request::builder()
        .method("GET").uri("/boards")
        .body(axum::body::Body::empty()).unwrap();
    let resp = ta.app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);
    let body = axum::body::to_bytes(resp.into_body(), 1024 * 1024).await.unwrap();
    let html = String::from_utf8(body.to_vec()).unwrap();
    assert!(html.contains("root"), "SPA must contain root div");
}

#[tokio::test]
async fn test_spa_serves_on_board_path() {
    let ta = setup().await;
    let req = axum::http::Request::builder()
        .method("GET").uri("/board/my-slug/my-board")
        .body(axum::body::Body::empty()).unwrap();
    let resp = ta.app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);
    let body = axum::body::to_bytes(resp.into_body(), 1024 * 1024).await.unwrap();
    let html = String::from_utf8(body.to_vec()).unwrap();
    assert!(html.contains("root"), "SPA must contain root div");
}

#[tokio::test]
async fn test_spa_serves_on_settings() {
    let ta = setup().await;
    let req = axum::http::Request::builder()
        .method("GET").uri("/settings")
        .body(axum::body::Body::empty()).unwrap();
    let resp = ta.app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);
    let body = axum::body::to_bytes(resp.into_body(), 1024 * 1024).await.unwrap();
    let html = String::from_utf8(body.to_vec()).unwrap();
    assert!(html.contains("root"), "SPA must contain root div");
}

// ── SPA assets are served correctly ───────────────────────────────────

#[tokio::test]
async fn test_spa_assets_are_served() {
    let ta = setup().await;
    // Vite builds assets to /assets/index-{hash}.js
    let req = axum::http::Request::builder()
        .method("GET").uri("/assets/")
        .body(axum::body::Body::empty()).unwrap();
    let resp = ta.app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);
}

#[tokio::test]
async fn test_spa_unknown_path_returns_index_html() {
    let ta = setup().await;
    let req = axum::http::Request::builder()
        .method("GET").uri("/some-unknown-path")
        .body(axum::body::Body::empty()).unwrap();
    let resp = ta.app.oneshot(req).await.unwrap();
    // SPA fallback: unknown paths serve index.html
    assert_eq!(resp.status(), 200);
    let body = axum::body::to_bytes(resp.into_body(), 1024 * 1024).await.unwrap();
    let html = String::from_utf8(body.to_vec()).unwrap();
    assert!(html.contains("root"), "SPA fallback should serve index.html");
}

// ── API still works ───────────────────────────────────────────────────

#[tokio::test]
async fn test_api_unauthenticated_returns_401() {
    let ta = setup().await;
    let req = axum::http::Request::builder()
        .method("GET").uri("/api/v1/boards")
        .body(axum::body::Body::empty()).unwrap();
    let resp = ta.app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 401);
}

#[tokio::test]
async fn test_api_authenticated_returns_200() {
    let ta = setup().await;
    let cookie = register(&ta.app).await;
    let req = axum::http::Request::builder()
        .method("GET").uri("/api/v1/boards")
        .header("cookie", &cookie)
        .body(axum::body::Body::empty()).unwrap();
    let resp = ta.app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);
}
