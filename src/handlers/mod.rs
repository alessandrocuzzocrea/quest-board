pub mod auth;
pub mod board;
pub mod card;
pub mod list;
pub mod label;
pub mod comment;
pub mod attachment;
pub mod favorite;
pub mod search;

use axum::Router;
use std::sync::Arc;

use crate::AppState;

/// Convenience: a user endpoint that lists users
pub fn user_router() -> Router<Arc<AppState>> {
    use axum::routing::get;
    Router::new().route("/", get(list_users))
}

use axum::extract::State;
use axum::Json;
use serde_json::json;

async fn list_users(
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, crate::error::AppError> {
    let users: Vec<crate::models::user::User> = sqlx::query_as("SELECT * FROM users ORDER BY name")
        .fetch_all(&state.db)
        .await?;

    let response: Vec<crate::models::user::UserResponse> = users.into_iter().map(Into::into).collect();
    Ok(Json(json!(response)))
}
