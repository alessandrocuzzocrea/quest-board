mod db;
mod error;
mod handlers;
mod models;

use std::sync::Arc;
use tower_sessions::cookie::SameSite;
use tower_sessions::MemoryStore;

pub struct AppState {
    pub db: sqlx::SqlitePool,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter("quest_board=debug")
        .init();

    let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:quest.db?mode=rwc".into());

    let pool = sqlx::SqlitePool::connect(&database_url)
        .await
        .expect("failed to connect to database");

    db::run_migrations(&pool).await.expect("failed to run migrations");

    let session_store = MemoryStore::default();
    let session_layer = tower_sessions::SessionManagerLayer::new(session_store)
        .with_secure(false)
        .with_same_site(SameSite::Lax);
    let state = Arc::new(AppState { db: pool });


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
        .nest("/users", handlers::user_router())
        .layer(tower_http::cors::CorsLayer::permissive())
        .with_state(state);

    let app = axum::Router::new()
        .nest("/api/v1", api)
        .layer(session_layer);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001").await.unwrap();
    tracing::info!("listening on http://0.0.0.0:3001");
    axum::serve(listener, app).await.unwrap();
}
