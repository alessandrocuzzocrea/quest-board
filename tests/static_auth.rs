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

// ── HTML content structure tests ───────────────────────────────

#[tokio::test]
async fn test_board_html_contains_kanban_and_modals() {
    let ta = setup().await;
    let cookie = register(&ta.app).await;

    let req = axum::http::Request::builder()
        .method("GET").uri("/board.html")
        .header("cookie", &cookie)
        .body(axum::body::Body::empty()).unwrap();
    let resp = ta.app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200, "board.html should serve successfully");
    let ct = resp.headers().get("content-type").and_then(|v| v.to_str().ok()).unwrap_or("").to_string();
    assert!(ct.contains("text/html"), "board.html should be served as text/html, got: {ct}");

    let body = axum::body::to_bytes(resp.into_body(), 1024 * 1024).await.unwrap();
    let html = String::from_utf8(body.to_vec()).unwrap();

    assert!(html.contains("kanban"), "board.html: missing kanban element");
    assert!(html.contains("card-modal"), "board.html: missing card-modal");
    assert!(html.contains("add-card-modal"), "board.html: missing add-card-modal");
    assert!(html.contains("add-list-modal"), "board.html: missing add-list-modal");
    assert!(html.contains("panel-title"), "board.html: missing panel-title");
    assert!(html.contains("modal-main"), "board.html: missing modal-main");
    assert!(html.contains("card-modal-sidebar"), "board.html: missing card-modal-sidebar");
    assert!(html.contains("draggable"), "board.html: kanban cards must be draggable");
    assert!(html.contains("ondragstart"), "board.html: missing ondragstart handler");
    assert!(html.contains("showLabelsPicker"), "board.html: sidebar missing showLabelsPicker");
    assert!(html.contains("archiveCard"), "board.html: sidebar missing archiveCard");
    assert!(html.contains("deleteCardFromPanel"), "board.html: sidebar missing deleteCardFromPanel");
    // Due/start date display on kanban cards
    assert!(html.contains("c.due_date"), "board.html: renderKanban must render due_date on cards");
    assert!(html.contains("c.start_date"), "board.html: renderKanban must render start_date on cards");
}

#[tokio::test]
async fn test_boards_html_contains_grid_and_modals() {
    let ta = setup().await;
    let cookie = register(&ta.app).await;

    let req = axum::http::Request::builder()
        .method("GET").uri("/boards.html")
        .header("cookie", &cookie)
        .body(axum::body::Body::empty()).unwrap();
    let resp = ta.app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);

    let body = axum::body::to_bytes(resp.into_body(), 1024 * 1024).await.unwrap();
    let html = String::from_utf8(body.to_vec()).unwrap();

    assert!(html.contains("board-grid"), "boards page must have a board grid");
    assert!(html.contains("create-modal"), "boards page must have create board modal");
    assert!(html.contains("delete-modal"), "boards page must have delete confirmation modal");
    assert!(html.contains("search-bar"), "boards page must have a search bar");
    assert!(html.contains("+ New Board"), "boards page must have a create button");
    assert!(html.contains("htmx"), "boards page uses htmx for search");
}

#[tokio::test]
async fn test_settings_html_contains_sections() {
    let ta = setup().await;
    let cookie = register(&ta.app).await;

    let req = axum::http::Request::builder()
        .method("GET").uri("/settings.html")
        .header("cookie", &cookie)
        .body(axum::body::Body::empty()).unwrap();
    let resp = ta.app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);

    let body = axum::body::to_bytes(resp.into_body(), 1024 * 1024).await.unwrap();
    let html = String::from_utf8(body.to_vec()).unwrap();

    assert!(html.contains("settings-section"), "settings must have section containers");
    assert!(html.contains("updateProfile"), "settings must have profile update form");
    assert!(html.contains("changePassword"), "settings must have password change form");
    assert!(html.contains("api-keys"), "settings must have API keys section");
}

#[tokio::test]
async fn test_css_contains_required_styles() {
    let ta = setup().await;

    let req = axum::http::Request::builder()
        .method("GET").uri("/css/style.css")
        .body(axum::body::Body::empty()).unwrap();
    let resp = ta.app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);

    let body = axum::body::to_bytes(resp.into_body(), 1024 * 1024).await.unwrap();
    let css = String::from_utf8(body.to_vec()).unwrap();

    // Core layout
    assert!(css.contains("kanban"), "CSS must define kanban layout");
    assert!(css.contains("column"), "CSS must define column styles");
    assert!(css.contains("modal-overlay"), "CSS must define modal styles");

    // Card detail modal v2
    assert!(css.contains("modal-card-v2"), "CSS must define v2 card modal layout");
    assert!(css.contains("card-modal-header"), "CSS must define card modal header");
    assert!(css.contains("card-modal-sidebar"), "CSS must define card modal sidebar");
    assert!(css.contains("task-progress-bar"), "CSS must define task progress bar");
    assert!(css.contains("sidebar-btn"), "CSS must define sidebar buttons");
    assert!(css.contains("comment-tabs"), "CSS must define comment tabs");
    // Kanban card date badges
    assert!(css.contains("date-badge"), "CSS must define date-badge styles");
    assert!(css.contains("date-start"), "CSS must define date-start style");
    assert!(css.contains("date-due"), "CSS must define date-due style");
    assert!(css.contains("date-overdue"), "CSS must define date-overdue style");

    // Responsive
    assert!(css.contains("@media"), "CSS must have responsive rules");
}

#[tokio::test]
async fn test_api_js_contains_all_endpoints() {
    let ta = setup().await;

    let req = axum::http::Request::builder()
        .method("GET").uri("/js/api.js")
        .body(axum::body::Body::empty()).unwrap();
    let resp = ta.app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);

    let body = axum::body::to_bytes(resp.into_body(), 1024 * 1024).await.unwrap();
    let js = String::from_utf8(body.to_vec()).unwrap();

    // Core API methods
    assert!(js.contains("getBoard"), "API must have getBoard");
    assert!(js.contains("createBoard"), "API must have createBoard");
    assert!(js.contains("getCard"), "API must have getCard");
    assert!(js.contains("createCard"), "API must have createCard");
    assert!(js.contains("moveCard"), "API must have moveCard");
    assert!(js.contains("addCardLabel"), "API must have addCardLabel");
    assert!(js.contains("createComment"), "API must have createComment");
    assert!(js.contains("updateTask"), "API must have updateTask");
    assert!(js.contains("listComments"), "API must have listComments");
    assert!(js.contains("listBoardLabels"), "API must have listBoardLabels");

    // Auth methods
    assert!(js.contains("login"), "API must have login");
    assert!(js.contains("logout"), "API must have logout");
    assert!(js.contains("requireAuth"), "API must have requireAuth helper");

    // DOM helpers
    assert!(js.contains("function $("), "must have DOM selector helper");
    assert!(js.contains("showAlert"), "must have alert helper");
}
