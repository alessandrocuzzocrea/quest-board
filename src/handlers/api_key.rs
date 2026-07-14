use axum::extract::{Path, State};
use axum::http::HeaderMap;
use axum::{Json, Router};
use std::sync::Arc;

use crate::auth::resolve_user;
use crate::error::AppError;
use crate::models::api_key::{ApiKeyResponse, CreateApiKeyRequest};
use crate::repository;
use crate::AppState;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", axum::routing::get(list_api_keys).post(create_api_key))
        .route("/{id}", axum::routing::delete(delete_api_key))
}

/// GET /api/v1/api-keys — lists all API keys for the authenticated user.
async fn list_api_keys(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, AppError> {
    let user_id = resolve_user(&session, &headers, &state.db).await?;
    let keys = repository::api_key_repo::list_by_user(&state.db, user_id).await?;
    let responses: Vec<ApiKeyResponse> =
        keys.into_iter().map(ApiKeyResponse::from).collect();
    Ok(Json(serde_json::json!(responses)))
}

/// POST /api/v1/api-keys — creates a new API key.
///
/// Returns the key metadata along with the raw token (shown only once).
async fn create_api_key(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    headers: HeaderMap,
    Json(req): Json<CreateApiKeyRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let user_id = resolve_user(&session, &headers, &state.db).await?;

    let (full_token, prefix, token_hash) = crate::auth::generate_api_key();

    let key = repository::api_key_repo::create(
        &state.db,
        user_id,
        &req.name,
        &token_hash,
        &prefix,
        req.expires_at,
    )
    .await?;

    let response = ApiKeyResponse::from(key);
    Ok(Json(serde_json::json!({
        "api_key": response,
        "token": full_token,
    })))
}

/// DELETE /api/v1/api-keys/{id} — soft-deletes (deactivates) an API key.
async fn delete_api_key(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    headers: HeaderMap,
    Path(key_id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let user_id = resolve_user(&session, &headers, &state.db).await?;

    let key_id = uuid::Uuid::parse_str(&key_id)
        .map_err(|_| AppError::BadRequest("invalid key id".into()))?;

    repository::api_key_repo::delete(&state.db, key_id, user_id).await?;

    Ok(Json(serde_json::json!({ "success": true })))
}
