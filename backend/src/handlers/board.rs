use axum::extract::{Path, State};
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

async fn user_id(session: &tower_sessions::Session) -> Result<uuid::Uuid, AppError> {
    let uid: String = session.get("user_id").await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or(AppError::Unauthorized("not logged in".into()))?;
    uuid::Uuid::parse_str(&uid).map_err(|_| AppError::Internal("invalid user id".into()))
}

#[utoipa::path(
    get,
    path = "/boards/",
    tag = "boards",
    responses(
        (status = 200, description = "List of boards", body = serde_json::Value),
        (status = 401, description = "Unauthorized")
    )
)]
 
async fn list_boards(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
) -> Result<Json<serde_json::Value>, AppError> {
    let uid = user_id(&session).await?;
    let boards = repository::board_repo::list_accessible(&state.db, &uid).await?;
    Ok(Json(serde_json::json!(boards)))
}

#[utoipa::path(
    post,
    path = "/boards/",
    tag = "boards",
    request_body = CreateBoardRequest,
    responses(
        (status = 200, description = "Board created", body = serde_json::Value),
        (status = 401, description = "Unauthorized")
    )
)]
 
async fn create_board(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    Json(req): Json<CreateBoardRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let uid = user_id(&session).await?;

    let board = repository::board_repo::create(&state.db, &req.name, &uid).await?;
    repository::board_repo::add_member(&state.db, &board.id, &uid, "admin").await?;
    repository::list_repo::create_defaults(&state.db, &board.id).await?;

    Ok(Json(serde_json::json!(board)))
}

#[utoipa::path(
    get,
    path = "/boards/{id}",
    tag = "boards",
    params(
        ("id" = String, Path, description = "Board ID")
    ),
    responses(
        (status = 200, description = "Board with lists and members", body = serde_json::Value),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Board not found")
    )
)]
 
async fn get_board(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    Path(board_id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let _uid = user_id(&session).await?;
    let board_id: uuid::Uuid = board_id.parse().map_err(|_| AppError::BadRequest("invalid id".into()))?;

    let (board, lists, members) = repository::board_repo::get_full_board(&state.db, &board_id).await?;

    Ok(Json(serde_json::json!({
        "board": board,
        "lists": lists,
        "members": members,
    })))
}

#[utoipa::path(
    put,
    path = "/boards/{id}",
    tag = "boards",
    request_body = UpdateBoardRequest,
    params(
        ("id" = String, Path, description = "Board ID")
    ),
    responses(
        (status = 200, description = "Board updated", body = serde_json::Value),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Board not found")
    )
)]
 
async fn update_board(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    Path(board_id): Path<String>,
    Json(req): Json<UpdateBoardRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let _uid = user_id(&session).await?;
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

#[utoipa::path(
    delete,
    path = "/boards/{id}",
    tag = "boards",
    params(
        ("id" = String, Path, description = "Board ID")
    ),
    responses(
        (status = 200, description = "Board deleted"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Board not found")
    )
)]
 
async fn delete_board(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    Path(board_id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let _uid = user_id(&session).await?;
    let board_id: uuid::Uuid = board_id.parse().map_err(|_| AppError::BadRequest("invalid id".into()))?;
    repository::board_repo::delete(&state.db, &board_id).await?;
    Ok(Json(serde_json::json!({"ok": true})))
}

#[utoipa::path(
    get,
    path = "/boards/by-slug/{slug}",
    tag = "boards",
    params(
        ("slug" = String, Path, description = "Board slug")
    ),
    responses(
        (status = 200, description = "Board with lists and members", body = serde_json::Value),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Board not found")
    )
)]
async fn get_board_by_slug(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    Path(slug): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let _uid = user_id(&session).await?;

    let (board, lists, members) = repository::board_repo::get_full_board_by_slug(&state.db, &slug).await?;

    Ok(Json(serde_json::json!({
        "board": board,
        "lists": lists,
        "members": members,
    })))
}
