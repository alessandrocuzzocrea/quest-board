pub mod db;
pub mod ai_tools;
pub mod auth;
pub mod error;
pub mod handlers;
pub mod models;
pub mod repository;
pub mod slug;
pub mod session;
use axum::routing::get;
use axum::{
    middleware,
    response::{Html, IntoResponse, Redirect},
};
use tower_http::services::fs::ServeDir;

use std::sync::Arc;
use tower_sessions::cookie::SameSite;

use session::PgSessionStore;

pub struct AppState {
    pub db: sqlx::PgPool,
    pub ai_client: Arc<dyn handlers::ai::LlmClient>,
}

// Known app pages that require authentication (both clean and .html forms)
const PROTECTED_PATHS: &[&str] = &[
    "/boards", "/board", "/settings",
    "/boards.html", "/board.html", "/settings.html",
];

async fn require_auth_for_html(
    request: axum::http::Request<axum::body::Body>,
    next: middleware::Next,
) -> axum::response::Response {

    let req_path = request.uri().path().to_string();

    if req_path == "/login" {
        return next.run(request).await;
    }

    let is_protected = req_path.ends_with(".html")
        || PROTECTED_PATHS.contains(&req_path.as_str());

    if !is_protected {
        return next.run(request).await;
    }

    // Check session for user_id
    let session = request.extensions()
        .get::<tower_sessions::Session>()
        .cloned();

    match session {
        Some(session) => {
            match session.get::<String>("user_id").await {
                Ok(Some(_)) => next.run(request).await,
                _ => Redirect::to("/login").into_response(),
            }
        }
        None => Redirect::to("/login").into_response(),
    }
}

// ── Page handlers for clean URLs ─────────────────────────────────────

async fn page_boards() -> impl IntoResponse {
    Html(include_str!("../static/boards.html"))
}

async fn page_board() -> impl IntoResponse {
    Html(include_str!("../static/board.html"))
}

async fn page_settings() -> impl IntoResponse {
    Html(include_str!("../static/settings.html"))
}

pub async fn build_app(pool: sqlx::PgPool, state: Arc<AppState>) -> axum::Router {
    let session_store = PgSessionStore::new(pool);
    let session_layer = tower_sessions::SessionManagerLayer::new(session_store)
        .with_secure(false)
        .with_same_site(SameSite::Lax);

    let app_state = state.clone();
    let api = axum::Router::new()
        .nest("/auth", handlers::auth::router())
        .nest("/boards", handlers::board::router())
        .nest("/lists", handlers::list::router())
        .nest("/cards", handlers::card::router())
        .nest("/labels", handlers::label::router())
        .nest("/comments", handlers::comment::router())
        .nest("/attachments", handlers::attachment::router())
        .nest("/favorites", handlers::favorite::router())
        .nest("/search", handlers::search::router())
        .nest("/health", handlers::health::router())
        .nest("/api-keys", handlers::api_key::router())
        .nest("/ai", handlers::ai::router())
        .nest("/users", handlers::user_router())
        .layer(tower_http::cors::CorsLayer::permissive())
        .with_state(state);

    let static_files = ServeDir::new("static");

    axum::Router::new()
        .route("/login", get(handlers::auth::htmx_login_page).post(handlers::auth::htmx_login))
        .route("/boards", get(page_boards))
        .route("/board", get(page_board))
        .route("/settings", get(page_settings))
        .nest("/api/v1", api)
        .fallback_service(static_files)
        .layer(middleware::from_fn(require_auth_for_html))
        .layer(session_layer)
        .with_state(app_state)
}

