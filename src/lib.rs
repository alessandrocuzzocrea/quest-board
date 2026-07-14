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

// ── Root path handler ────────────────────────────────────────────────

async fn root_handler(
    session: tower_sessions::Session,
) -> impl IntoResponse {
    match session.get::<String>("user_id").await {
        Ok(Some(_)) => Redirect::to("/boards"),
        _ => Redirect::to("/login"),
    }
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
        .route("/", get(root_handler))
        .route("/boards", get(page_boards))
        .route("/board", get(page_board))
        .route("/settings", get(page_settings))
        .nest("/api/v1", api)
        .fallback_service(static_files)
        .layer(middleware::from_fn(require_auth_for_html))
        .layer(session_layer)
        .with_state(app_state)
}


// ── CLI integration ─────────────────────────────────────────────
pub mod cli {
    /// Returns a hello message showing the backend URL the CLI connects to.
    pub fn hello_msg(backend_url: &str) -> String {
        format!("Hello from quest-board CLI! Backend: {backend_url}")
    }

    /// Greet and print backend health status (or error).
    pub async fn run(backend_url: &str) -> Result<String, String> {
        let msg = hello_msg(backend_url);
        match reqwest::get(&format!("{backend_url}/health")).await {
            Ok(resp) => {
                let status = resp.status();
                let text = resp.text().await.unwrap_or_default();
                Ok(format!("{msg}\nBackend status: {status}\n{text}"))
            }
            Err(e) => Ok(format!("{msg}\nCould not reach backend: {e}")),
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_hello_msg_format() {
            let msg = hello_msg("http://localhost:3000");
            assert!(msg.contains("Hello from quest-board CLI!"));
            assert!(msg.contains("http://localhost:3000"));
            assert_eq!(msg, "Hello from quest-board CLI! Backend: http://localhost:3000");
        }

        #[tokio::test]
        async fn test_run_returns_greeting() {
            // Should not panic even with unreachable backend
            let result = run("http://localhost:1").await;
            assert!(result.is_ok());
            let output = result.unwrap();
            assert!(output.contains("Hello from quest-board CLI!"));
            assert!(output.contains("Could not reach backend"));
        }
    }
}
