use axum::extract::{Path, State};
use axum::http::HeaderMap;
use axum::{Json, Router};
use std::sync::Arc;

use crate::auth::resolve_user;
use crate::error::AppError;
use crate::models::api_key::CreateApiKeyRequest;
use crate::services::ApiKeyService;
use crate::AppState;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", axum::routing::get(list_api_keys).post(create_api_key))
        .route("/{id}", axum::routing::delete(delete_api_key))
}

/// GET /api/v1/api-keys — lists all API keys for the authenticated user.
#[utoipa::path(get, path = "/api/v1/api-keys", tag = "api-keys", responses((status = 200, body = serde_json::Value)))]
async fn list_api_keys(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, AppError> {
    let user_id = resolve_user(&session, &headers, &state.db).await?;
    let svc = ApiKeyService::new(state.db.clone());
    let keys = svc.list_by_user(user_id).await?;
    Ok(Json(serde_json::json!(keys)))
}

/// POST /api/v1/api-keys — creates a new API key.
///
/// Returns the key metadata along with the raw token (shown only once).
#[utoipa::path(
    post,
    path = "/api/v1/api-keys",
    tag = "api-keys",
    request_body = CreateApiKeyRequest,
    responses(
        (status = 200, body = serde_json::Value)
    )
)]
async fn create_api_key(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    headers: HeaderMap,
    Json(req): Json<CreateApiKeyRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let user_id = resolve_user(&session, &headers, &state.db).await?;
    let svc = ApiKeyService::new(state.db.clone());
    let (response, token) = svc.create(user_id, &req).await?;
    Ok(Json(serde_json::json!({
        "api_key": response,
        "token": token,
    })))
}

/// DELETE /api/v1/api-keys/{id} — soft-deletes (deactivates) an API key.
#[utoipa::path(
    delete,
    path = "/api/v1/api-keys/{id}",
    tag = "api-keys",
    params(("id" = String, Path)),
    responses(
        (status = 200, body = serde_json::Value)
    )
)]
async fn delete_api_key(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    headers: HeaderMap,
    Path(key_id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let user_id = resolve_user(&session, &headers, &state.db).await?;
    let key_id = uuid::Uuid::parse_str(&key_id)
        .map_err(|_| AppError::BadRequest("invalid key id".into()))?;
    let svc = ApiKeyService::new(state.db.clone());
    svc.delete(key_id, user_id).await?;
    Ok(Json(serde_json::json!({ "success": true })))
}
