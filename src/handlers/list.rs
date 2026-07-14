use axum::extract::{Path, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use std::sync::Arc;

use crate::error::AppError;
use crate::models::list::*;
use crate::repository;
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

 
async fn create_list(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    Json(req): Json<CreateListRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let uid = user_id(&session).await?;
    let list = repository::list_repo::create(&state.db, &req.board_id, req.name.as_deref().unwrap_or("New List"), 65536.0).await?;
    crate::events::emit_simple(&state.event_tx, "list_created", &req.board_id.to_string(),
        None, Some(&list.id.to_string()), &uid.to_string());
    Ok(Json(serde_json::json!(list)))
}
 
async fn get_list(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    Path(list_id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let _uid = user_id(&session).await?;
    let list_id: uuid::Uuid = list_id.parse().map_err(|_| AppError::BadRequest("invalid list id".into()))?;

    let list = repository::list_repo::get_by_id(&state.db, &list_id)
        .await?
        .ok_or(AppError::NotFound("list not found".into()))?;

    let cards = sqlx::query_as::<_, crate::models::card::Card>(
        "SELECT * FROM cards WHERE list_id = $1 ORDER BY position, created_at",
    )
    .bind(&list_id)
    .fetch_all(&state.db)
    .await?;

    Ok(Json(serde_json::json!({"list": list, "cards": cards})))
}

 
async fn update_list(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    Path(list_id): Path<String>,
    Json(req): Json<UpdateListRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let uid = user_id(&session).await?;
    let list_id: uuid::Uuid = list_id.parse().map_err(|_| AppError::BadRequest("invalid list id".into()))?;
    if let Some(name) = &req.name {
        repository::list_repo::update_name(&state.db, &list_id, name).await?;
    }
    if let Some(position) = req.position {
        repository::list_repo::update_position(&state.db, &list_id, position).await?;
    }
    if let Some(color) = &req.color {
        repository::list_repo::update_color(&state.db, &list_id, color).await?;
    }
    let list = repository::list_repo::get_by_id(&state.db, &list_id)
        .await?
        .ok_or(AppError::NotFound("list not found".into()))?;
    crate::events::emit_simple(&state.event_tx, "list_updated", &list.board_id.to_string(),
        None, Some(&list.id.to_string()), &uid.to_string());
    Ok(Json(serde_json::json!(list)))
}

async fn delete_list(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    Path(list_id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let uid = user_id(&session).await?;
    let list_id: uuid::Uuid = list_id.parse().map_err(|_| AppError::BadRequest("invalid list id".into()))?;
    // Get list for board_id before deletion
    let list = repository::list_repo::get_by_id(&state.db, &list_id).await.unwrap();
    let board_id = list.as_ref().map(|l| l.board_id.to_string()).unwrap_or_default();
    repository::list_repo::delete(&state.db, &list_id).await?;
    crate::events::emit_simple(&state.event_tx, "list_deleted", &board_id,
        None, Some(&list_id.to_string()), &uid.to_string());
    Ok(Json(serde_json::json!({"ok": true})))
}
