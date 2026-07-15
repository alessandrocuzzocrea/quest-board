use askama::Template;
use axum::extract::{Query, State};
use axum::http::HeaderMap;
use axum::response::{Html, IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use serde::Deserialize;
use std::sync::Arc;

use crate::error::AppError;
use crate::models::board::Board;
use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    q: String,
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/", get(search))
}

async fn search(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    Query(query): Query<SearchQuery>,
    headers: HeaderMap,
) -> Result<Response, AppError> {
    let user_id: String = session.get("user_id").await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or(AppError::Unauthorized("not logged in".into()))?;
    let uid = uuid::Uuid::parse_str(&user_id).map_err(|_| AppError::Internal("invalid user id".into()))?;

    let pattern = format!("%{}%", query.q);
    let is_htmx = headers.contains_key("HX-Request");

    if is_htmx {
        // HTMX path: return HTML partial with board cards
        let boards: Vec<Board> = sqlx::query_as(
            "SELECT DISTINCT b.* FROM boards b
             LEFT JOIN board_members bm ON b.id = bm.board_id AND bm.user_id = $1
             WHERE b.name LIKE $2 AND (b.created_by = $3 OR bm.user_id = $4)
             ORDER BY b.name
             LIMIT 10",
        )
        .bind(uid)
        .bind(&pattern)
        .bind(uid)
        .bind(uid)
        .fetch_all(&state.db)
        .await?;

        let template = crate::BoardGridTemplate { boards, query: query.q };
        Ok(Html(template.render().map_err(|e| AppError::Internal(e.to_string()))?).into_response())
    } else {
        // JSON API path: return cards and boards as JSON
        let cards: Vec<serde_json::Value> = sqlx::query_as::<_, (uuid::Uuid, String, uuid::Uuid, String, f64)>(
            "SELECT c.id, c.name, c.board_id, l.name as list_name, c.position
             FROM cards c
             JOIN lists l ON c.list_id = l.id
             JOIN boards b ON c.board_id = b.id
             LEFT JOIN board_members bm ON b.id = bm.board_id AND bm.user_id = $1
             WHERE (c.name LIKE $2 OR c.description LIKE $3)
               AND (b.created_by = $4 OR bm.user_id = $5)
             ORDER BY c.position
             LIMIT 20",
        )
        .bind(uid)
        .bind(&pattern)
        .bind(&pattern)
        .bind(uid)
        .bind(uid)
        .fetch_all(&state.db)
        .await?
        .into_iter()
        .map(|(id, name, board_id, list_name, _pos)| {
            serde_json::json!({"id": id, "name": name, "board_id": board_id, "list_name": list_name})
        })
        .collect();

        let boards: Vec<serde_json::Value> = sqlx::query_as::<_, (uuid::Uuid, String)>(
            "SELECT DISTINCT b.id, b.name
             FROM boards b
             LEFT JOIN board_members bm ON b.id = bm.board_id AND bm.user_id = $1
             WHERE b.name LIKE $2 AND (b.created_by = $3 OR bm.user_id = $4)
             ORDER BY b.name
             LIMIT 10",
        )
        .bind(uid)
        .bind(&pattern)
        .bind(uid)
        .bind(uid)
        .fetch_all(&state.db)
        .await?
        .into_iter()
        .map(|(id, name)| serde_json::json!({"id": id, "name": name}))
        .collect();

        Ok(Json(serde_json::json!({"cards": cards, "boards": boards})).into_response())
    }
}
