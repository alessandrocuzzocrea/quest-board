use axum::extract::{Path, State};
use axum::routing::{get, post, put};
use axum::{Json, Router};
use std::sync::Arc;

use crate::error::AppError;
use crate::models::label::{CreateLabelRequest, UpdateLabelRequest};
use crate::services::LabelService;
use crate::AppState;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", post(create_label))
        .route("/board/{board_id}", get(list_labels))
        .route("/{id}", put(update_label).delete(delete_label))
}

async fn user_id(session: &tower_sessions::Session) -> Result<uuid::Uuid, AppError> {
    let uid: String = session.get("user_id").await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or(AppError::Unauthorized("not logged in".into()))?;
    uuid::Uuid::parse_str(&uid).map_err(|_| AppError::Internal("invalid user id".into()))
}

#[utoipa::path(
    get,
    path = "/api/v1/labels/board/{board_id}",
    tag = "labels",
    params(("board_id" = String, Path)),
    responses(
        (status = 200, description = "List of labels", body = serde_json::Value)
    )
)]
async fn list_labels(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    Path(board_id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let _uid = user_id(&session).await?;
    let board_id: uuid::Uuid = board_id.parse().map_err(|_| AppError::BadRequest("invalid board id".into()))?;
    let svc = LabelService::new(state.db.clone());
    let labels = svc.list_by_board(&board_id).await?;
    Ok(Json(serde_json::json!(labels)))
}

#[utoipa::path(
    post,
    path = "/api/v1/labels",
    tag = "labels",
    request_body = CreateLabelRequest,
    responses(
        (status = 200, description = "Label created", body = serde_json::Value)
    )
)]
async fn create_label(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    Json(req): Json<CreateLabelRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let _uid = user_id(&session).await?;
    let svc = LabelService::new(state.db.clone());
    let label = svc.create(&req).await?;
    Ok(Json(serde_json::json!(label)))
}

#[utoipa::path(
    put,
    path = "/api/v1/labels/{id}",
    tag = "labels",
    params(("id" = String, Path)),
    request_body = UpdateLabelRequest,
    responses(
        (status = 200, description = "Label updated", body = serde_json::Value)
    )
)]
async fn update_label(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    Path(label_id): Path<String>,
    Json(req): Json<UpdateLabelRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let _uid = user_id(&session).await?;
    let label_id: uuid::Uuid = label_id.parse().map_err(|_| AppError::BadRequest("invalid label id".into()))?;
    let svc = LabelService::new(state.db.clone());
    let label = svc.update(&label_id, &req).await?;
    Ok(Json(serde_json::json!(label)))
}

#[utoipa::path(
    delete,
    path = "/api/v1/labels/{id}",
    tag = "labels",
    params(("id" = String, Path)),
    responses(
        (status = 200, description = "Label deleted", body = serde_json::Value)
    )
)]
async fn delete_label(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    Path(label_id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let _uid = user_id(&session).await?;
    let label_id: uuid::Uuid = label_id.parse().map_err(|_| AppError::BadRequest("invalid label id".into()))?;
    let svc = LabelService::new(state.db.clone());
    svc.delete(&label_id).await?;
    Ok(Json(serde_json::json!({"ok": true})))
}
