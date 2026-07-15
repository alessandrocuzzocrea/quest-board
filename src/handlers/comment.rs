use axum::extract::{Path, State};
use axum::routing::{get, post, put};
use axum::{Json, Router};
use std::sync::Arc;

use crate::error::AppError;
use crate::models::comment::{CreateCommentRequest, UpdateCommentRequest};
use crate::services::CommentService;
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
    let svc = CommentService::new(state.db.clone(), state.event_tx.clone());
    let comments = svc.list_by_card(&card_id).await?;
    Ok(Json(serde_json::json!(comments)))
}

async fn create_comment(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    Json(req): Json<CreateCommentRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let uid = user_id(&session).await?;
    let svc = CommentService::new(state.db.clone(), state.event_tx.clone());
    let comment = svc.create(&req, &uid).await?;
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
    let svc = CommentService::new(state.db.clone(), state.event_tx.clone());
    let comment = svc.update(&comment_id, &req, &uid).await?;
    Ok(Json(serde_json::json!(comment)))
}

async fn delete_comment(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    Path(comment_id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let uid = user_id(&session).await?;
    let comment_id: uuid::Uuid = comment_id.parse().map_err(|_| AppError::BadRequest("invalid comment id".into()))?;
    let svc = CommentService::new(state.db.clone(), state.event_tx.clone());
    svc.delete(&comment_id, &uid).await?;
    Ok(Json(serde_json::json!({"ok": true})))
}
