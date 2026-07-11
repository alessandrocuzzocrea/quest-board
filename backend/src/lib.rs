pub mod db;
pub mod auth;
pub mod error;
pub mod handlers;
pub mod models;
pub mod repository;
pub mod slug;
pub mod session;

use std::sync::Arc;
use tower_http::services::fs::ServeDir;
use tower_sessions::cookie::SameSite;

use session::PgSessionStore;

pub struct AppState {
    pub db: sqlx::PgPool,
}

pub async fn build_app(pool: sqlx::PgPool, state: Arc<AppState>) -> axum::Router {
    let session_store = PgSessionStore::new(pool);
    let session_layer = tower_sessions::SessionManagerLayer::new(session_store)
        .with_secure(false)
        .with_same_site(SameSite::Lax);

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
        .nest("/api-keys", handlers::api_key::router())
        .nest("/users", handlers::user_router())
        .layer(tower_http::cors::CorsLayer::permissive())
        .with_state(state);

    let static_files = ServeDir::new("static").not_found_service(
        tower_http::services::fs::ServeFile::new("static/index.html"),
    );

    axum::Router::new()
        .nest("/api/v1", api)
        .fallback_service(static_files)
        .layer(session_layer)
}
