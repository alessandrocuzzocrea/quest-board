use axum::extract::{Path, State};
use axum::routing::{get, delete};
use axum::{Json, Router};
use std::sync::Arc;

use crate::error::AppError;
use crate::models::favorite::*;
use crate::AppState;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(list_favorites).post(create_favorite))
        .route("/{id}", delete(delete_favorite))
}

async fn require_user(session: &tower_sessions::Session) -> Result<String, AppError> {
    session
        .get::<String>("user_id")
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or(AppError::Unauthorized("not logged in".into()))
}

async fn list_favorites(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
) -> Result<Json<serde_json::Value>, AppError> {
    let user_id = require_user(&session).await?;

    let favorites: Vec<Favorite> = sqlx::query_as(
        "SELECT * FROM favorites WHERE user_id = $1 ORDER BY created_at",
    )
    .bind(&user_id)
    .fetch_all(&state.db)
    .await?;

    let mut boards: Vec<serde_json::Value> = Vec::new();
    let mut cards: Vec<serde_json::Value> = Vec::new();

    for f in favorites {
        if let Some(board_id) = f.board_id {
            let board: Option<crate::models::board::Board> = sqlx::query_as("SELECT * FROM boards WHERE id = $1")
                .bind(&board_id)
                .fetch_optional(&state.db)
                .await?;
            if let Some(b) = board {
                boards.push(serde_json::json!({
                    "board_id": b.id,
                    "name": b.name,
                }));
            }
        }
        if let Some(card_id_val) = f.card_id {
            let card: Option<(String, String)> = sqlx::query_as(
                "SELECT id, name FROM cards WHERE id = $1",
            )
            .bind(&card_id_val)
            .fetch_optional(&state.db)
            .await?;
            if let Some((cid, cname)) = card {
                cards.push(serde_json::json!({
                    "card_id": cid,
                    "name": cname,
                }));
            }
        }
    }

    Ok(Json(serde_json::json!({
        "boards": boards,
        "cards": cards,
    })))
}

async fn create_favorite(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    Json(req): Json<CreateFavoriteRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let user_id = require_user(&session).await?;
    let id = uuid::Uuid::new_v4().to_string();

    sqlx::query(
        "INSERT OR IGNORE INTO favorites (id, user_id, board_id, card_id) VALUES ($1, $2, $3, $4)",
    )
    .bind(&id)
    .bind(&user_id)
    .bind(&req.board_id)
    .bind(&req.card_id)
    .execute(&state.db)
    .await?;

    Ok(Json(serde_json::json!({"ok": true})))
}

async fn delete_favorite(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    Path(fav_id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let _user_id = require_user(&session).await?;
    sqlx::query("DELETE FROM favorites WHERE id = $1")
        .bind(&fav_id)
        .execute(&state.db)
        .await?;
    Ok(Json(serde_json::json!({"ok": true})))
}
