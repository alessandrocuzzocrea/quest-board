use axum::extract::{Json, State};
use axum::routing::{get, post, put};
use axum::Router;
use axum::http::HeaderMap;
use std::sync::Arc;

use crate::error::AppError;
use crate::models::user::*;
use crate::services::AuthService;
use crate::AppState;

/// Core login logic shared by JSON and HTMX handlers.
async fn try_login(
    state: &Arc<AppState>,
    session: &tower_sessions::Session,
    req: &LoginRequest,
) -> Result<crate::models::user::User, AppError> {
    let svc = AuthService::new(state.db.clone());
    let user = svc.login(&req.username, &req.password).await?;
    session.insert("user_id", &user.id).await.map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(user)
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/register", post(register))
        .route("/me", get(me))
        .route("/login", post(login))
        .route("/logout", post(logout))
        .route("/me/password", put(change_password))
}

#[utoipa::path(post, path = "/api/v1/auth/register", tag = "auth", request_body = RegisterRequest, responses((status = 200, body = serde_json::Value)))]
async fn register(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    Json(req): Json<RegisterRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let svc = AuthService::new(state.db.clone());
    let user = svc.register(&req.username, &req.password).await?;
    session.insert("user_id", user.id.to_string()).await.map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(Json(serde_json::json!({"user": {"id": user.id.to_string(), "username": req.username, "role": "user"}})))
}

#[utoipa::path(post, path = "/api/v1/auth/login", tag = "auth", request_body = LoginRequest, responses((status = 200, body = serde_json::Value)))]
async fn login(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    Json(req): Json<LoginRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let user = try_login(&state, &session, &req).await?;
    Ok(Json(serde_json::json!({"user": UserResponse::from(user)})))
}

async fn logout(session: tower_sessions::Session) -> Result<Json<serde_json::Value>, AppError> {
    session.flush().await.map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(Json(serde_json::json!({"ok": true})))
}

#[utoipa::path(get, path = "/api/v1/auth/me", tag = "auth", responses((status = 200, body = serde_json::Value)))]
async fn me(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, AppError> {
    let uid = crate::auth::resolve_user(&session, &headers, &state.db).await?;
    let svc = AuthService::new(state.db.clone());
    let user = svc.get_user(&uid).await?;
    Ok(Json(serde_json::json!({"user": UserResponse::from(user)})))
}

#[utoipa::path(put, path = "/api/v1/auth/me/password", tag = "auth", request_body = ChangePasswordRequest, responses((status = 200, body = serde_json::Value)))]
async fn change_password(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    headers: HeaderMap,
    Json(req): Json<ChangePasswordRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let uid = crate::auth::resolve_user(&session, &headers, &state.db).await?;
    let svc = AuthService::new(state.db.clone());
    svc.change_password(&uid, &req.current_password, &req.new_password).await?;
    Ok(Json(serde_json::json!({"ok": true})))
}

