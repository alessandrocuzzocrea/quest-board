use askama::Template;
use axum::extract::{Path, State};
use axum::http::HeaderMap;
use axum::response::{Html, IntoResponse};
use axum::routing::get;
use axum::{Json, Router};
use std::sync::Arc;

use crate::error::AppError;
use crate::models::board::{CreateBoardRequest, UpdateBoardRequest};
use crate::services::BoardService;
use crate::AppState;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(list_boards).post(create_board))
        .route("/{id}", get(get_board).put(update_board).delete(delete_board))
        .route("/by-slug/{slug}", get(get_board_by_slug))
        .route("/html", get(list_boards_html))
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
    let svc = BoardService::new(state.db.clone());
    let boards = svc.list_accessible(&uid).await?;
    Ok(Json(serde_json::json!(boards)))
}

/// HTML endpoint — returns board cards as HTML for htmx `hx-trigger="load"`.
async fn list_boards_html(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    headers: HeaderMap,
) -> Result<impl IntoResponse, AppError> {
    let uid = user_id(session, headers, &state.db).await?;
    let svc = BoardService::new(state.db.clone());
    let boards = svc.list_accessible(&uid).await?;
    let tmpl = crate::BoardGridTemplate { boards, query: String::new() };
    Ok(Html(
        tmpl.render().map_err(|e| AppError::Internal(e.to_string()))?,
    ))
}

async fn create_board(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    headers: HeaderMap,
    Json(req): Json<CreateBoardRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let uid = user_id(session, headers, &state.db).await?;
    let svc = BoardService::new(state.db.clone());
    let board = svc.create(&req.name, &uid).await?;
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
    let svc = BoardService::new(state.db.clone());
    let (board, lists, members) = svc.get_full(&board_id).await?;
    Ok(Json(serde_json::json!({"board": board, "lists": lists, "members": members})))
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
    let svc = BoardService::new(state.db.clone());
    let board = svc.update(&board_id, req.name.as_deref(), req.position).await?;
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
    let svc = BoardService::new(state.db.clone());
    svc.delete(&board_id).await?;
    Ok(Json(serde_json::json!({"ok": true})))
}

async fn get_board_by_slug(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    headers: HeaderMap,
    Path(slug): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let _uid = user_id(session, headers, &state.db).await?;
    let svc = BoardService::new(state.db.clone());
    let (board, lists, members) = svc.get_full_by_slug(&slug).await?;
    Ok(Json(serde_json::json!({"board": board, "lists": lists, "members": members})))
}
