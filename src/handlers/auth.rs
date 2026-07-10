use argon2::password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use argon2::Argon2;
use axum::extract::State;
use axum::routing::post;
use axum::{Json, Router};
use password_hash::rand_core::OsRng;
use std::sync::Arc;

use crate::error::AppError;
use crate::models::user::*;
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

    let existing: Option<User> = sqlx::query_as("SELECT * FROM users WHERE email = ?")
        .bind(&req.email)
        .fetch_optional(&state.db)
        .await?;
    if existing.is_some() {
        return Err(AppError::BadRequest("email already registered".into()));
    }

    let salt = SaltString::generate(&mut OsRng);
    let password_hash = Argon2::default()
        .hash_password(req.password.as_bytes(), &salt)
        .map_err(|_| AppError::Internal("failed to hash password".into()))?
        .to_string();

    let id = uuid::Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO users (id, email, password_hash, name) VALUES (?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(&req.email)
    .bind(&password_hash)
    .bind(&req.name)
    .execute(&state.db)
    .await?;

    session
        .insert("user_id", &id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(Json(serde_json::json!({
        "user": {
            "id": id,
            "email": req.email,
            "name": req.name,
            "role": "user",
        }
    })))
}

async fn login(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    Json(req): Json<LoginRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let user: Option<User> = sqlx::query_as("SELECT * FROM users WHERE email = ?")
        .bind(&req.email)
        .fetch_optional(&state.db)
        .await?;

    let user = user.ok_or(AppError::Unauthorized("invalid email or password".into()))?;

    let parsed_hash = PasswordHash::new(&user.password_hash)
        .map_err(|_| AppError::Internal("auth error".into()))?;

    Argon2::default()
        .verify_password(req.password.as_bytes(), &parsed_hash)
        .map_err(|_| AppError::Unauthorized("invalid email or password".into()))?;

    session
        .insert("user_id", &user.id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(Json(serde_json::json!({
        "user": UserResponse::from(user)
    })))
}

async fn logout(
    session: tower_sessions::Session,
) -> Result<Json<serde_json::Value>, AppError> {
    session
        .flush()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(Json(serde_json::json!({"ok": true})))
}

async fn me(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
) -> Result<Json<serde_json::Value>, AppError> {
    let user_id: Option<String> = session
        .get("user_id")
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let user_id = user_id.ok_or(AppError::Unauthorized("not logged in".into()))?;

    let user: User = sqlx::query_as("SELECT * FROM users WHERE id = ?")
        .bind(&user_id)
        .fetch_optional(&state.db)
        .await?
        .ok_or(AppError::Unauthorized("user not found".into()))?;

    Ok(Json(serde_json::json!({
        "user": UserResponse::from(user)
    })))
}
