use axum::extract::{Path, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use std::sync::Arc;

use crate::error::AppError;
use crate::models::list::*;
use crate::AppState;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", post(create_list))
        .route("/{id}", get(get_list).put(update_list).delete(delete_list))
}

async fn require_user(session: &tower_sessions::Session) -> Result<String, AppError> {
    session
        .get::<String>("user_id")
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or(AppError::Unauthorized("not logged in".into()))
}

async fn create_list(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    Json(req): Json<CreateListRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let _user_id = require_user(&session).await?;
    let id = uuid::Uuid::new_v4().to_string();

    sqlx::query(
        "INSERT INTO lists (id, board_id, name, position) VALUES ($1, $2, $3, $4)",
    )
    .bind(&id)
    .bind(&req.board_id)
    .bind(&req.name)
    .bind(65536.0f64)
    .execute(&state.db)
    .await?;

    let list: List = sqlx::query_as("SELECT * FROM lists WHERE id = $1")
        .bind(&id)
        .fetch_one(&state.db)
        .await?;

    Ok(Json(serde_json::json!(list)))
}

async fn get_list(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    Path(list_id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let _user_id = require_user(&session).await?;

    let list: List = sqlx::query_as("SELECT * FROM lists WHERE id = $1")
        .bind(&list_id)
        .fetch_optional(&state.db)
        .await?
        .ok_or(AppError::NotFound("list not found".into()))?;

    let cards: Vec<crate::models::card::Card> = sqlx::query_as(
        "SELECT * FROM cards WHERE list_id = $1 ORDER BY position, created_at",
    )
    .bind(&list_id)
    .fetch_all(&state.db)
    .await?;

    Ok(Json(serde_json::json!({
        "list": list,
        "cards": cards,
    })))
}

async fn update_list(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    Path(list_id): Path<String>,
    Json(req): Json<UpdateListRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let _user_id = require_user(&session).await?;

    if let Some(name) = &req.name {
        sqlx::query("UPDATE lists SET name = $1, updated_at = NOW() WHERE id = $2")
            .bind(name)
            .bind(&list_id)
            .execute(&state.db)
            .await?;
    }
    if let Some(position) = req.position {
        sqlx::query("UPDATE lists SET position = $1, updated_at = NOW() WHERE id = $2")
            .bind(position)
            .bind(&list_id)
            .execute(&state.db)
            .await?;
    }
    if let Some(color) = &req.color {
        sqlx::query("UPDATE lists SET color = $1, updated_at = NOW() WHERE id = $2")
            .bind(color)
            .bind(&list_id)
            .execute(&state.db)
            .await?;
    }

    let list: List = sqlx::query_as("SELECT * FROM lists WHERE id = $1")
        .bind(&list_id)
        .fetch_optional(&state.db)
        .await?
        .ok_or(AppError::NotFound("list not found".into()))?;

    Ok(Json(serde_json::json!(list)))
}

async fn delete_list(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    Path(list_id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let _user_id = require_user(&session).await?;
    sqlx::query("DELETE FROM lists WHERE id = $1")
        .bind(&list_id)
        .execute(&state.db)
        .await?;
    Ok(Json(serde_json::json!({"ok": true})))
}
