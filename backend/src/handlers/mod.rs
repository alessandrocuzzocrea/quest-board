pub mod auth;
pub mod attachment;
pub mod board;
pub mod card;
pub mod comment;
pub mod favorite;
pub mod label;
pub mod list;
pub mod search;

use axum::extract::State;
use axum::routing::get;
use axum::{Json, Router};
use std::sync::Arc;

use crate::repository;
use crate::AppState;

pub fn user_router() -> Router<Arc<AppState>> {
    Router::new().route("/", get(list_users))
}

async fn list_users(
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, crate::error::AppError> {
    let users = repository::user_repo::list_all(&state.db).await?;
    Ok(Json(serde_json::json!(users)))
}
