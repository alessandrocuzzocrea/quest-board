use askama::Template;
use axum::extract::{Query, State};
use axum::http::HeaderMap;
use axum::response::{Html, IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use serde::Deserialize;
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

    let svc = SearchService::new(state.db.clone());
    let is_htmx = headers.contains_key("HX-Request");

    if is_htmx {
        // HTMX path: return HTML partial with board cards
        let boards = svc.search_boards(&uid, &query.q).await?;
        let template = crate::BoardGridTemplate { boards, query: query.q };
        Ok(Html(template.render().map_err(|e| AppError::Internal(e.to_string()))?).into_response())
    } else {
        // JSON API path: return cards and boards as JSON
        let cards = svc.search_cards(&uid, &query.q).await?;
        let boards_json: Vec<serde_json::Value> = {
            // Reuse search_boards but map to flat JSON for API compat
            let boards = svc.search_boards(&uid, &query.q).await?;
            boards.into_iter().map(|b| {
                serde_json::json!({"id": b.id, "name": b.name})
            }).collect()
        };

        Ok(Json(serde_json::json!({"cards": cards, "boards": boards_json})).into_response())
    }
}
