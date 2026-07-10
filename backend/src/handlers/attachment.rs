use axum::extract::{Path, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use std::sync::Arc;

use crate::error::AppError;
use crate::repository;
use crate::AppState;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/card/{card_id}", get(list_attachments))
        .route("/link", post(create_link_attachment))
        .route("/{id}", axum::routing::delete(delete_attachment))
}

async fn user_id(session: &tower_sessions::Session) -> Result<String, AppError> {
    session.get("user_id").await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or(AppError::Unauthorized("not logged in".into()))
}

async fn list_attachments(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    Path(card_id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let _uid = user_id(&session).await?;
    let attachments = repository::attachment_repo::list_by_card(&state.db, &card_id).await?;
    Ok(Json(serde_json::json!(attachments)))
}

async fn create_link_attachment(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    let uid = user_id(&session).await?;
    let id = uuid::Uuid::new_v4().to_string();

    let card_id = req["card_id"].as_str().ok_or(AppError::BadRequest("card_id required".into()))?;
    let name = req["name"].as_str().unwrap_or("link");
    let url = req["url"].as_str().ok_or(AppError::BadRequest("url required".into()))?;

    let attachment = repository::attachment_repo::create_link(&state.db, &id, card_id, &uid, name, url).await?;
    Ok(Json(serde_json::json!(attachment)))
}

async fn delete_attachment(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    Path(attachment_id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let _uid = user_id(&session).await?;
    repository::attachment_repo::delete(&state.db, &attachment_id).await?;
    Ok(Json(serde_json::json!({"ok": true})))
}
