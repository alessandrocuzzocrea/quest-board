use axum::extract::{Path, State};
use axum::routing::{get, post, put};
use axum::{Json, Router};
use std::sync::Arc;

use crate::error::AppError;
use crate::models::label::*;
use crate::AppState;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", post(create_label))
        .route("/board/{board_id}", get(list_labels))
        .route("/{id}", put(update_label).delete(delete_label))
}

async fn require_user(session: &tower_sessions::Session) -> Result<String, AppError> {
    session
        .get::<String>("user_id")
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or(AppError::Unauthorized("not logged in".into()))
}

async fn list_labels(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    Path(board_id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let _user_id = require_user(&session).await?;

    let labels: Vec<Label> = sqlx::query_as(
        "SELECT * FROM labels WHERE board_id = ? ORDER BY position, name",
    )
    .bind(&board_id)
    .fetch_all(&state.db)
    .await?;

    Ok(Json(serde_json::json!(labels)))
}

async fn create_label(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    Json(req): Json<CreateLabelRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let _user_id = require_user(&session).await?;
    let id = uuid::Uuid::new_v4().to_string();

    sqlx::query(
        "INSERT INTO labels (id, board_id, name, color, position) VALUES (?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(&req.board_id)
    .bind(&req.name)
    .bind(&req.color.unwrap_or_else(|| "#0079bf".into()))
    .bind(65536.0f64)
    .execute(&state.db)
    .await?;

    let label: Label = sqlx::query_as("SELECT * FROM labels WHERE id = ?")
        .bind(&id)
        .fetch_one(&state.db)
        .await?;

    Ok(Json(serde_json::json!(label)))
}

async fn update_label(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    Path(label_id): Path<String>,
    Json(req): Json<UpdateLabelRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let _user_id = require_user(&session).await?;

    if let Some(ref name) = req.name {
        sqlx::query("UPDATE labels SET name = ?, updated_at = datetime('now') WHERE id = ?")
            .bind(name)
            .bind(&label_id)
            .execute(&state.db)
            .await?;
    }
    if let Some(ref color) = req.color {
        sqlx::query("UPDATE labels SET color = ?, updated_at = datetime('now') WHERE id = ?")
            .bind(color)
            .bind(&label_id)
            .execute(&state.db)
            .await?;
    }
    if let Some(position) = req.position {
        sqlx::query("UPDATE labels SET position = ?, updated_at = datetime('now') WHERE id = ?")
            .bind(position)
            .bind(&label_id)
            .execute(&state.db)
            .await?;
    }

    let label: Label = sqlx::query_as("SELECT * FROM labels WHERE id = ?")
        .bind(&label_id)
        .fetch_optional(&state.db)
        .await?
        .ok_or(AppError::NotFound("label not found".into()))?;

    Ok(Json(serde_json::json!(label)))
}

async fn delete_label(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    Path(label_id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let _user_id = require_user(&session).await?;
    sqlx::query("DELETE FROM labels WHERE id = ?")
        .bind(&label_id)
        .execute(&state.db)
        .await?;
    Ok(Json(serde_json::json!({"ok": true})))
}
