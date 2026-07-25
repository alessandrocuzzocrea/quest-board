use serde::Deserialize;
use axum::extract::{Query, State};
use axum::routing::get;
use axum::{Json, Router};
use std::sync::Arc;

use crate::error::AppError;
use crate::services::SearchService;
use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    q: String,
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/", get(search))
}

#[utoipa::path(
    get,
    path = "/api/v1/search",
    tag = "search",
    params(("q" = String, Query)),
    responses(
        (status = 200, body = serde_json::Value)
    )
)]
async fn search(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    Query(query): Query<SearchQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    let user_id: String = session.get("user_id").await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or(AppError::Unauthorized("not logged in".into()))?;
    let uid = uuid::Uuid::parse_str(&user_id).map_err(|_| AppError::Internal("invalid user id".into()))?;

    let svc = SearchService::new(state.db.clone());

    let cards = svc.search_cards(&uid, &query.q).await?;
    let board_results = svc.search_boards(&uid, &query.q).await?;
    let boards_json: Vec<serde_json::Value> = board_results.into_iter().map(|b| {
        serde_json::json!({"id": b.id, "name": b.name})
    }).collect();

    Ok(Json(serde_json::json!({"cards": cards, "boards": boards_json})))
}
