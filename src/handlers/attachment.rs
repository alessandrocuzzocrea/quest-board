use axum::extract::{Path, State};
use axum::routing::{get, post, delete};
use axum::{Json, Router};
use std::sync::Arc;

use crate::error::AppError;
use crate::models::attachment::*;
use crate::AppState;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/card/{card_id}", get(list_attachments))
        .route("/link", post(create_link_attachment))
        .route("/{id}", delete(delete_attachment))
}

async fn require_user(session: &tower_sessions::Session) -> Result<String, AppError> {
    session
        .get::<String>("user_id")
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or(AppError::Unauthorized("not logged in".into()))
}

async fn list_attachments(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    Path(card_id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let _user_id = require_user(&session).await?;

    let attachments: Vec<Attachment> = sqlx::query_as(
        "SELECT * FROM attachments WHERE card_id = $1 ORDER BY created_at",
    )
    .bind(&card_id)
    .fetch_all(&state.db)
    .await?;

    Ok(Json(serde_json::json!(attachments)))
}

async fn create_link_attachment(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    let user_id = require_user(&session).await?;
    let id = uuid::Uuid::new_v4().to_string();

    let card_id = req["card_id"].as_str().ok_or(AppError::BadRequest("card_id required".into()))?;
    let name = req["name"].as_str().unwrap_or("link");
    let link_url = req["url"].as_str().ok_or(AppError::BadRequest("url required".into()))?;

    sqlx::query(
        "INSERT INTO attachments (id, card_id, user_id, name, attachment_type, link_url) VALUES ($1, $2, $3, $4, 'link', $5)",
    )
    .bind(&id)
    .bind(card_id)
    .bind(&user_id)
    .bind(name)
    .bind(link_url)
    .execute(&state.db)
    .await?;

    let attachment: Attachment = sqlx::query_as("SELECT * FROM attachments WHERE id = $1")
        .bind(&id)
        .fetch_one(&state.db)
        .await?;

    Ok(Json(serde_json::json!(attachment)))
}

async fn delete_attachment(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    Path(attachment_id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let _user_id = require_user(&session).await?;
    sqlx::query("DELETE FROM attachments WHERE id = $1")
        .bind(&attachment_id)
        .execute(&state.db)
        .await?;
    Ok(Json(serde_json::json!({"ok": true})))
}
