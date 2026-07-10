use axum::extract::{Path, State};
use axum::routing::get;
use axum::{Json, Router};
use std::sync::Arc;

use crate::error::AppError;
use crate::models::favorite::CreateFavoriteRequest;
use crate::repository;
use crate::AppState;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(list_favorites).post(create_favorite))
        .route("/{id}", axum::routing::delete(delete_favorite))
}

async fn user_id(session: &tower_sessions::Session) -> Result<String, AppError> {
    session.get("user_id").await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or(AppError::Unauthorized("not logged in".into()))
}

async fn list_favorites(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
) -> Result<Json<serde_json::Value>, AppError> {
    let uid = user_id(&session).await?;
    let result = repository::favorite_repo::list_by_user(&state.db, &uid).await?;
    Ok(Json(result))
}

async fn create_favorite(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    Json(req): Json<CreateFavoriteRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let uid = user_id(&session).await?;
    repository::favorite_repo::create(&state.db, &uid, req.board_id.as_deref(), req.card_id.as_deref()).await?;
    Ok(Json(serde_json::json!({"ok": true})))
}

async fn delete_favorite(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    Path(fav_id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let _uid = user_id(&session).await?;
    repository::favorite_repo::delete(&state.db, &fav_id).await?;
    Ok(Json(serde_json::json!({"ok": true})))
}
