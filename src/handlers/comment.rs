use axum::extract::{Path, State};
use axum::routing::{get, post, put};
use axum::{Json, Router};
use std::sync::Arc;

use crate::error::AppError;
use crate::models::comment::*;
use crate::repository;
use crate::AppState;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", post(create_comment))
        .route("/card/{card_id}", get(list_comments))
        .route("/{id}", put(update_comment).delete(delete_comment))
}

async fn user_id(session: &tower_sessions::Session) -> Result<uuid::Uuid, AppError> {
    let uid: String = session.get("user_id").await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or(AppError::Unauthorized("not logged in".into()))?;
    uuid::Uuid::parse_str(&uid).map_err(|_| AppError::Internal("invalid user id".into()))
}

 
async fn list_comments(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    Path(card_id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let _uid = user_id(&session).await?;
    let card_id: uuid::Uuid = card_id.parse().map_err(|_| AppError::BadRequest("invalid card id".into()))?;
    let comments = repository::comment_repo::list_by_card(&state.db, &card_id).await?;
    Ok(Json(serde_json::json!(comments)))
}

 
async fn create_comment(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    Json(req): Json<CreateCommentRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let uid = user_id(&session).await?;
    let now = chrono::Utc::now().to_rfc3339();

    let comment = repository::comment_repo::create(&state.db, &req.card_id, &uid, &req.text, &now).await?;

    repository::action_repo::record(
        &state.db, &req.card_id, Some(&uid), "commentCard",
        serde_json::json!({"comment": {"text": &req.text}}),
    ).await?;

    // Emit event (no board_id available here without extra query)
    crate::events::emit_simple(&state.event_tx, "comment_created", "",
        Some(&req.card_id.to_string()), None, &uid.to_string());

    Ok(Json(serde_json::json!(comment)))
}

 
async fn update_comment(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    Path(comment_id): Path<String>,
    Json(req): Json<UpdateCommentRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let uid = user_id(&session).await?;
    let comment_id: uuid::Uuid = comment_id.parse().map_err(|_| AppError::BadRequest("invalid comment id".into()))?;
    let comment = repository::comment_repo::update_text(&state.db, &comment_id, &req.text).await?;
    crate::events::emit_simple(&state.event_tx, "comment_updated", "",
        Some(&comment.card_id.to_string()), None, &uid.to_string());
    Ok(Json(serde_json::json!(comment)))
}

async fn delete_comment(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    Path(comment_id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let uid = user_id(&session).await?;
    let comment_id: uuid::Uuid = comment_id.parse().map_err(|_| AppError::BadRequest("invalid comment id".into()))?;
    // Fetch comment for card_id before deletion
    let comment = repository::comment_repo::get_by_id(&state.db, &comment_id).await.unwrap();
    let card_id = comment.as_ref().map(|c| c.card_id.to_string()).unwrap_or_default();
    repository::comment_repo::delete(&state.db, &comment_id).await?;
    crate::events::emit_simple(&state.event_tx, "comment_deleted", "",
        Some(&card_id), None, &uid.to_string());
    Ok(Json(serde_json::json!({"ok": true})))
}
