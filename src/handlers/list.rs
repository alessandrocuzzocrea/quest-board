use axum::extract::{Path, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use std::sync::Arc;

use crate::error::AppError;
use crate::models::list::{CreateListRequest, UpdateListRequest};
use crate::services::ListService;
use crate::AppState;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", post(create_list))
        .route("/{id}", get(get_list).put(update_list).delete(delete_list))
}

async fn user_id(session: &tower_sessions::Session) -> Result<uuid::Uuid, AppError> {
    let uid: String = session.get("user_id").await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or(AppError::Unauthorized("not logged in".into()))?;
    uuid::Uuid::parse_str(&uid).map_err(|_| AppError::Internal("invalid user id".into()))
}

#[utoipa::path(
    post,
    path = "/api/v1/lists",
    tag = "lists",
    request_body = CreateListRequest,
    responses(
        (status = 200, description = "List created", body = serde_json::Value)
    )
)]

async fn create_list(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    Json(req): Json<CreateListRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let uid = user_id(&session).await?;
    let svc = ListService::new(state.db.clone(), state.event_tx.clone());
    let list = svc.create(&req, &uid).await?;
    Ok(Json(serde_json::json!(list)))
}

#[utoipa::path(
    get,
    path = "/api/v1/lists/{id}",
    tag = "lists",
    params(("id" = String, Path)),
    responses(
        (status = 200, description = "List retrieved", body = serde_json::Value)
    )
)]

async fn get_list(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    Path(list_id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let _uid = user_id(&session).await?;
    let list_id: uuid::Uuid = list_id.parse().map_err(|_| AppError::BadRequest("invalid list id".into()))?;
    let svc = ListService::new(state.db.clone(), state.event_tx.clone());
    let (list, cards) = svc.get_with_cards(&list_id).await?;
    Ok(Json(serde_json::json!({"list": list, "cards": cards})))
}

#[utoipa::path(
    put,
    path = "/api/v1/lists/{id}",
    tag = "lists",
    params(("id" = String, Path)),
    request_body = UpdateListRequest,
    responses(
        (status = 200, description = "List updated", body = serde_json::Value)
    )
)]

async fn update_list(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    Path(list_id): Path<String>,
    Json(req): Json<UpdateListRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let uid = user_id(&session).await?;
    let list_id: uuid::Uuid = list_id.parse().map_err(|_| AppError::BadRequest("invalid list id".into()))?;
    let svc = ListService::new(state.db.clone(), state.event_tx.clone());
    let list = svc.update(&list_id, &req, &uid).await?;
    Ok(Json(serde_json::json!(list)))
}

#[utoipa::path(
    delete,
    path = "/api/v1/lists/{id}",
    tag = "lists",
    params(("id" = String, Path)),
    responses(
        (status = 200, description = "List deleted", body = serde_json::Value)
    )
)]

async fn delete_list(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    Path(list_id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let uid = user_id(&session).await?;
    let list_id: uuid::Uuid = list_id.parse().map_err(|_| AppError::BadRequest("invalid list id".into()))?;
    let svc = ListService::new(state.db.clone(), state.event_tx.clone());
    svc.delete(&list_id, &uid).await?;
    Ok(Json(serde_json::json!({"ok": true})))
}
