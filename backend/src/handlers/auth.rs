use argon2::password_hash::{PasswordHasher, PasswordVerifier};
use argon2::password_hash::phc::PasswordHash;
use argon2::Argon2;
use axum::extract::State;
use axum::routing::post;
use axum::{Json, Router};
use std::sync::Arc;

use crate::error::AppError;
use crate::models::user::*;
use crate::repository;
use crate::AppState;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/logout", post(logout))
        .route("/me", axum::routing::get(me))
}

async fn register(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    Json(req): Json<RegisterRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    if req.email.is_empty() || req.password.is_empty() || req.name.is_empty() {
        return Err(AppError::BadRequest("email, password, and name are required".into()));
    }

    if repository::user_repo::find_by_email(&state.db, &req.email).await?.is_some() {
        return Err(AppError::BadRequest("email already registered".into()));
    }

    let pw_hash = Argon2::default()
        .hash_password(req.password.as_bytes())
        .map_err(|_| AppError::Internal("failed to hash password".into()))?
        .to_string();

    let user = repository::user_repo::create(&state.db, &req.email, &pw_hash, &req.name).await?;
    session.insert("user_id", user.id.to_string()).await.map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(Json(serde_json::json!({"user": {"id": user.id.to_string(), "email": req.email, "name": req.name, "role": "user"}})))
}

async fn login(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    Json(req): Json<LoginRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let user = repository::user_repo::find_by_email(&state.db, &req.email)
        .await?
        .ok_or(AppError::Unauthorized("invalid email or password".into()))?;

    let parsed = PasswordHash::new(&user.password_hash)
        .map_err(|_| AppError::Internal("auth error".into()))?;

    Argon2::default()
        .verify_password(req.password.as_bytes(), &parsed)
        .map_err(|_| AppError::Unauthorized("invalid email or password".into()))?;

    session.insert("user_id", &user.id).await.map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(Json(serde_json::json!({"user": UserResponse::from(user)})))
}

async fn logout(session: tower_sessions::Session) -> Result<Json<serde_json::Value>, AppError> {
    session.flush().await.map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(Json(serde_json::json!({"ok": true})))
}

async fn me(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
) -> Result<Json<serde_json::Value>, AppError> {
    let user_id_str: String = session.get("user_id").await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or(AppError::Unauthorized("not logged in".into()))?;
    let uid = uuid::Uuid::parse_str(&user_id_str).map_err(|_| AppError::Internal("invalid user id".into()))?;
    let user = repository::user_repo::find_by_id(&state.db, &uid)
        .await?
        .ok_or(AppError::Unauthorized("user not found".into()))?;

    Ok(Json(serde_json::json!({"user": UserResponse::from(user)})))
}
