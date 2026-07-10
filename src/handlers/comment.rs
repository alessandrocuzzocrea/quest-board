use axum::extract::{Path, State};
use axum::routing::{get, post, put};
use axum::{Json, Router};
use std::sync::Arc;

use crate::error::AppError;
use crate::models::comment::*;
use crate::AppState;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", post(create_comment))
        .route("/card/{card_id}", get(list_comments))
        .route("/{id}", put(update_comment).delete(delete_comment))
}

async fn require_user(session: &tower_sessions::Session) -> Result<String, AppError> {
    session
        .get::<String>("user_id")
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or(AppError::Unauthorized("not logged in".into()))
}

async fn list_comments(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    Path(card_id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let _user_id = require_user(&session).await?;

    let comments: Vec<Comment> = sqlx::query_as(
        "SELECT * FROM comments WHERE card_id = ? ORDER BY created_at",
    )
    .bind(&card_id)
    .fetch_all(&state.db)
    .await?;

    let mut result: Vec<CommentWithUser> = Vec::new();
    for c in comments {
        let user: Option<crate::models::user::User> = sqlx::query_as("SELECT * FROM users WHERE id = ?")
            .bind(&c.user_id)
            .fetch_optional(&state.db)
            .await?;

        result.push(CommentWithUser {
            id: c.id,
            card_id: c.card_id,
            user_id: c.user_id.clone(),
            user: user.map(Into::into),
            text: c.text,
            created_at: c.created_at,
            updated_at: c.updated_at,
        });
    }

    Ok(Json(serde_json::json!(result)))
}

async fn create_comment(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    Json(req): Json<CreateCommentRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let user_id = require_user(&session).await?;
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    sqlx::query("INSERT INTO comments (id, card_id, user_id, text, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)")
        .bind(&id)
        .bind(&req.card_id)
        .bind(&user_id)
        .bind(&req.text)
        .bind(&now)
        .bind(&now)
        .execute(&state.db)
        .await?;

    // Update card's updated_at
    sqlx::query("UPDATE cards SET updated_at = ? WHERE id = ?")
        .bind(&now)
        .bind(&req.card_id)
        .execute(&state.db)
        .await?;

    // Record action
    sqlx::query("INSERT INTO actions (id, card_id, user_id, action_type, data, created_at) VALUES (?, ?, ?, 'commentCard', ?, ?)")
        .bind(uuid::Uuid::new_v4().to_string())
        .bind(&req.card_id)
        .bind(&user_id)
        .bind(serde_json::json!({"comment": {"text": &req.text}}).to_string())
        .bind(&now)
        .execute(&state.db)
        .await?;

    let comment: Comment = sqlx::query_as("SELECT * FROM comments WHERE id = ?")
        .bind(&id)
        .fetch_one(&state.db)
        .await?;

    Ok(Json(serde_json::json!(comment)))
}

async fn update_comment(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    Path(comment_id): Path<String>,
    Json(req): Json<UpdateCommentRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let _user_id = require_user(&session).await?;

    sqlx::query("UPDATE comments SET text = ?, updated_at = datetime('now') WHERE id = ?")
        .bind(&req.text)
        .bind(&comment_id)
        .execute(&state.db)
        .await?;

    let comment: Comment = sqlx::query_as("SELECT * FROM comments WHERE id = ?")
        .bind(&comment_id)
        .fetch_optional(&state.db)
        .await?
        .ok_or(AppError::NotFound("comment not found".into()))?;

    Ok(Json(serde_json::json!(comment)))
}

async fn delete_comment(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    Path(comment_id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let _user_id = require_user(&session).await?;
    sqlx::query("DELETE FROM comments WHERE id = ?")
        .bind(&comment_id)
        .execute(&state.db)
        .await?;
    Ok(Json(serde_json::json!({"ok": true})))
}
