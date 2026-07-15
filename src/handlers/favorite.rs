use axum::extract::{Path, State};
use axum::routing::get;
use axum::{Json, Router};
use std::sync::Arc;

use crate::error::AppError;
use crate::models::favorite::CreateFavoriteRequest;
use crate::services::FavoriteService;
use crate::AppState;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(list_favorites).post(create_favorite))
        .route("/{id}", axum::routing::delete(delete_favorite))
}

async fn user_id(session: &tower_sessions::Session) -> Result<uuid::Uuid, AppError> {
    let uid: String = session.get("user_id").await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or(AppError::Unauthorized("not logged in".into()))?;
    uuid::Uuid::parse_str(&uid).map_err(|_| AppError::Internal("invalid user id".into()))
}

#[utoipa::path(get, path = "/api/v1/favorites", tag = "favorites", responses((status = 200, body = serde_json::Value)))]
async fn list_favorites(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
) -> Result<Json<serde_json::Value>, AppError> {
    let uid = user_id(&session).await?;
    let svc = FavoriteService::new(state.db.clone());
    let result = svc.list_by_user(&uid).await?;
    Ok(Json(result))
}

#[utoipa::path(post, path = "/api/v1/favorites", tag = "favorites", request_body = CreateFavoriteRequest, responses((status = 200, body = serde_json::Value)))]
async fn create_favorite(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    Json(req): Json<CreateFavoriteRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let uid = user_id(&session).await?;
    let svc = FavoriteService::new(state.db.clone());
    svc.create(&uid, req.board_id.as_ref(), req.card_id.as_ref()).await?;
    Ok(Json(serde_json::json!({"ok": true})))
}

#[utoipa::path(
    delete,
    path = "/api/v1/favorites/{id}",
    tag = "favorites",
    params(("id" = String, Path)),
    responses(
        (status = 200, body = serde_json::Value)
    )
)]
async fn delete_favorite(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    Path(fav_id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let _uid = user_id(&session).await?;
    let fid: uuid::Uuid = fav_id.parse().map_err(|_| AppError::BadRequest("invalid id".into()))?;
    let svc = FavoriteService::new(state.db.clone());
    svc.delete(&fid).await?;
    Ok(Json(serde_json::json!({"ok": true})))
}
