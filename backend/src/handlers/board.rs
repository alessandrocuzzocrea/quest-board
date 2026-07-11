use axum::extract::{Path, State};
use axum::http::HeaderMap;
use axum::routing::get;
use axum::{Json, Router};
use std::sync::Arc;

use crate::error::AppError;
use crate::models::board::*;
use crate::repository;
use crate::AppState;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(list_boards).post(create_board))
        .route("/{id}", get(get_board).put(update_board).delete(delete_board))
        .route("/by-slug/{slug}", get(get_board_by_slug))
}

async fn user_id(session: tower_sessions::Session, headers: HeaderMap, pool: &sqlx::PgPool) -> Result<uuid::Uuid, AppError> {
    crate::auth::resolve_user(&session, &headers, pool).await
}

async fn list_boards(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, AppError> {
    let uid = user_id(session, headers, &state.db).await?;
    let boards = repository::board_repo::list_accessible(&state.db, &uid).await?;
    Ok(Json(serde_json::json!(boards)))
}

async fn create_board(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    headers: HeaderMap,
    Json(req): Json<CreateBoardRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let uid = user_id(session, headers, &state.db).await?;

    let board = repository::board_repo::create(&state.db, &req.name, &uid).await?;
    repository::board_repo::add_member(&state.db, &board.id, &uid, "admin").await?;
    repository::list_repo::create_defaults(&state.db, &board.id).await?;

    Ok(Json(serde_json::json!(board)))
}

async fn get_board(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    headers: HeaderMap,
    Path(board_id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let _uid = user_id(session, headers, &state.db).await?;
    let board_id: uuid::Uuid = board_id.parse().map_err(|_| AppError::BadRequest("invalid id".into()))?;

    let (board, lists, members) = repository::board_repo::get_full_board(&state.db, &board_id).await?;

    Ok(Json(serde_json::json!({
        "board": board,
        "lists": lists,
        "members": members,
    })))
}

async fn update_board(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    headers: HeaderMap,
    Path(board_id): Path<String>,
    Json(req): Json<UpdateBoardRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let _uid = user_id(session, headers, &state.db).await?;
    let board_id: uuid::Uuid = board_id.parse().map_err(|_| AppError::BadRequest("invalid id".into()))?;

    if let Some(name) = &req.name {
        repository::board_repo::update_name(&state.db, &board_id, name).await?;
    }
    if let Some(position) = req.position {
        repository::board_repo::update_position(&state.db, &board_id, position).await?;
    }

    let board = repository::board_repo::get_by_id(&state.db, &board_id)
        .await?
        .ok_or(AppError::NotFound("board not found".into()))?;

    Ok(Json(serde_json::json!(board)))
}

async fn delete_board(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    headers: HeaderMap,
    Path(board_id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let _uid = user_id(session, headers, &state.db).await?;
    let board_id: uuid::Uuid = board_id.parse().map_err(|_| AppError::BadRequest("invalid id".into()))?;
    repository::board_repo::delete(&state.db, &board_id).await?;
    Ok(Json(serde_json::json!({"ok": true})))
}

async fn get_board_by_slug(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    headers: HeaderMap,
    Path(slug): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let _uid = user_id(session, headers, &state.db).await?;

    let (board, lists, members) = repository::board_repo::get_full_board_by_slug(&state.db, &slug).await?;

    Ok(Json(serde_json::json!({
        "board": board,
        "lists": lists,
        "members": members,
    })))
}
