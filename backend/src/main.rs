use std::sync::Arc;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter("quest_board=debug")
        .init();

    dotenvy::dotenv().ok();

    if std::env::var("APP_SECRET").unwrap_or_default().is_empty() {
        tracing::warn!("APP_SECRET not set — password hashing uses no pepper. Set it in .env for production.");
    }

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:quest@localhost:5432/quest".into());

    let pool = sqlx::PgPool::connect(&database_url)
        .await
        .expect("failed to connect to database");

    quest_board::db::run_migrations(&pool)
        .await
        .expect("failed to run migrations");

    let state = Arc::new(quest_board::AppState { db: pool.clone() });
    let app = quest_board::build_app(pool, state).await;

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001").await.unwrap();
    tracing::info!("listening on http://0.0.0.0:3001");
    axum::serve(listener, app).await.unwrap();
}
