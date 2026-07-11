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

async fn user_id(session: &tower_sessions::Session) -> Result<uuid::Uuid, AppError> {
    let uid: String = session.get("user_id").await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or(AppError::Unauthorized("not logged in".into()))?;
    uuid::Uuid::parse_str(&uid).map_err(|_| AppError::Internal("invalid user id".into()))
}

#[utoipa::path(
    get,
    path = "/attachments/card/{card_id}",
    tag = "attachments",
    params(
        ("card_id" = String, Path, description = "Card ID")
    ),
    responses(
        (status = 200, description = "List of attachments", body = serde_json::Value),
        (status = 401, description = "Unauthorized")
    )
)]
 
async fn list_attachments(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    Path(card_id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let _uid = user_id(&session).await?;
    let card_id: uuid::Uuid = card_id.parse().map_err(|_| AppError::BadRequest("invalid card_id".into()))?;
    let attachments = repository::attachment_repo::list_by_card(&state.db, &card_id).await?;
    Ok(Json(serde_json::json!(attachments)))
}

#[utoipa::path(
    post,
    path = "/attachments/link",
    tag = "attachments",
    request_body(content = serde_json::Value, description = "Link attachment data"),
    responses(
        (status = 200, description = "Link attachment created", body = serde_json::Value),
        (status = 401, description = "Unauthorized")
    )
)]
 
async fn create_link_attachment(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    let uid = user_id(&session).await?;
    let card_id_str = req["card_id"].as_str().ok_or(AppError::BadRequest("card_id required".into()))?;
    let card_id: uuid::Uuid = card_id_str.parse().map_err(|_| AppError::BadRequest("invalid card_id".into()))?;
    let name = req["name"].as_str().unwrap_or("link");
    let url = req["url"].as_str().ok_or(AppError::BadRequest("url required".into()))?;

    let attachment = repository::attachment_repo::create_link(&state.db, &card_id, &uid, name, url).await?;
    Ok(Json(serde_json::json!(attachment)))
}

#[utoipa::path(
    delete,
    path = "/attachments/{id}",
    tag = "attachments",
    params(
        ("id" = String, Path, description = "Attachment ID")
    ),
    responses(
        (status = 200, description = "Attachment deleted"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Attachment not found")
    )
)]
async fn delete_attachment(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    Path(attachment_id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let _uid = user_id(&session).await?;
    let attachment_id: uuid::Uuid = attachment_id.parse().map_err(|_| AppError::BadRequest("invalid attachment_id".into()))?;
    repository::attachment_repo::delete(&state.db, &attachment_id).await?;
    Ok(Json(serde_json::json!({"ok": true})))
}
