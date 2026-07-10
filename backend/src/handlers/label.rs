use axum::extract::{Path, State};
use axum::routing::{get, post, put};
use axum::{Json, Router};
use std::sync::Arc;

use crate::error::AppError;
use crate::models::label::*;
use crate::repository;
use crate::AppState;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", post(create_label))
        .route("/board/{board_id}", get(list_labels))
        .route("/{id}", put(update_label).delete(delete_label))
}

async fn user_id(session: &tower_sessions::Session) -> Result<String, AppError> {
    session.get("user_id").await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or(AppError::Unauthorized("not logged in".into()))
}

async fn list_labels(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    Path(board_id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let _uid = user_id(&session).await?;
    let labels = repository::label_repo::list_by_board(&state.db, &board_id).await?;
    Ok(Json(serde_json::json!(labels)))
}

async fn create_label(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    Json(req): Json<CreateLabelRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let _uid = user_id(&session).await?;
    let id = uuid::Uuid::new_v4().to_string();
    let label = repository::label_repo::create(
        &state.db, &id, &req.board_id, &req.name,
        &req.color.unwrap_or_else(|| "#0079bf".into()),
        65536.0,
    ).await?;
    Ok(Json(serde_json::json!(label)))
}

async fn update_label(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    Path(label_id): Path<String>,
    Json(req): Json<UpdateLabelRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let _uid = user_id(&session).await?;

    if let Some(name) = &req.name {
        repository::label_repo::update_name(&state.db, &label_id, name).await?;
    }
    if let Some(color) = &req.color {
        repository::label_repo::update_color(&state.db, &label_id, color).await?;
    }
    if let Some(position) = req.position {
        repository::label_repo::update_position(&state.db, &label_id, position).await?;
    }

    let label = repository::label_repo::get_by_id(&state.db, &label_id)
        .await?
        .ok_or(AppError::NotFound("label not found".into()))?;

    Ok(Json(serde_json::json!(label)))
}

async fn delete_label(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    Path(label_id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let _uid = user_id(&session).await?;
    repository::label_repo::delete(&state.db, &label_id).await?;
    Ok(Json(serde_json::json!({"ok": true})))
}
