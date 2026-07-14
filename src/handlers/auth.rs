use argon2::password_hash::{PasswordHasher, PasswordVerifier};
use argon2::password_hash::phc::PasswordHash;
use argon2::Argon2;
use axum::extract::{Form, State};
use axum::http::HeaderMap;
use axum::response::{Html, IntoResponse, Response};
use axum::routing::{get, post, put};
use axum::{Json, Router};
use axum::http::StatusCode;
use std::sync::Arc;

use crate::error::AppError;
use crate::models::user::*;
use crate::repository;
use crate::AppState;

fn pepper() -> String {
    crate::db::pepper()
}

async fn resolve(session: tower_sessions::Session, headers: HeaderMap, pool: &sqlx::PgPool) -> Result<uuid::Uuid, AppError> {
    crate::auth::resolve_user(&session, &headers, pool).await
}

/// Core login logic shared by JSON and HTMX handlers.
async fn try_login(
    state: &Arc<AppState>,
    session: &tower_sessions::Session,
    req: &LoginRequest,
) -> Result<crate::models::user::User, AppError> {
    let user = repository::user_repo::find_by_username(&state.db, &req.username)
        .await?
        .ok_or(AppError::Unauthorized("invalid username or password".into()))?;

    let parsed = PasswordHash::new(&user.password_hash)
        .map_err(|_| AppError::Internal("auth error".into()))?;

    let peppered = format!("{}{}", pepper(), req.password);
    Argon2::default()
        .verify_password(peppered.as_bytes(), &parsed)
        .map_err(|_| AppError::Unauthorized("invalid username or password".into()))?;

    session.insert("user_id", &user.id).await.map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(user)
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/logout", post(logout))
        .route("/me", get(me).put(update_name))
        .route("/me/password", put(change_password))
}

async fn register(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    Json(req): Json<RegisterRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    if req.username.is_empty() || req.password.is_empty() || req.name.is_empty() {
        return Err(AppError::BadRequest("username, password, and name are required".into()));
    }

    if repository::user_repo::find_by_username(&state.db, &req.username).await?.is_some() {
        return Err(AppError::BadRequest("username already registered".into()));
    }

    let peppered = format!("{}{}", pepper(), req.password);
    let pw_hash = Argon2::default()
        .hash_password(peppered.as_bytes())
        .map_err(|_| AppError::Internal("failed to hash password".into()))?
        .to_string();

    let user = repository::user_repo::create(&state.db, &req.username, &pw_hash, &req.name).await?;
    session.insert("user_id", user.id.to_string()).await.map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(Json(serde_json::json!({"user": {"id": user.id.to_string(), "username": req.username, "name": req.name, "role": "user"}})))
}

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

async fn me(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, AppError> {
    let uid = resolve(session, headers, &state.db).await?;
    let user = repository::user_repo::find_by_id(&state.db, &uid)
        .await?
        .ok_or(AppError::NotFound("user not found".into()))?;
    Ok(Json(serde_json::json!({"user": UserResponse::from(user)})))
}

async fn update_name(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    headers: HeaderMap,
    Json(req): Json<UpdateNameRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let uid = resolve(session, headers, &state.db).await?;
    let user = repository::user_repo::update_name(&state.db, &uid, &req.name).await?;
    Ok(Json(serde_json::json!({"user": UserResponse::from(user)})))
}

async fn change_password(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    headers: HeaderMap,
    Json(req): Json<ChangePasswordRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let uid = resolve(session, headers, &state.db).await?;
    let user = repository::user_repo::find_by_id(&state.db, &uid)
        .await?
        .ok_or(AppError::NotFound("user not found".into()))?;

    let parsed = PasswordHash::new(&user.password_hash)
        .map_err(|_| AppError::Internal("auth error".into()))?;

    let peppered = format!("{}{}", pepper(), req.old_password);
    Argon2::default()
        .verify_password(peppered.as_bytes(), &parsed)
        .map_err(|_| AppError::Unauthorized("invalid password".into()))?;

    let new_peppered = format!("{}{}", pepper(), req.new_password);
    let new_hash = Argon2::default()
        .hash_password(new_peppered.as_bytes())
        .map_err(|_| AppError::Internal("failed to hash password".into()))?
        .to_string();

    repository::user_repo::update_password(&state.db, &uid, &new_hash).await?;
    Ok(Json(serde_json::json!({"ok": true})))
}

/// Serves the login page for HTMX-based browsing.
pub async fn htmx_login_page() -> impl IntoResponse {
    Html(LOGIN_PAGE)
}

/// HTMX form login handler — accepts `application/x-www-form-urlencoded`.
pub async fn htmx_login(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    Form(req): Form<LoginRequest>,
) -> Response {
    match try_login(&state, &session, &req).await {
        Ok(_) => {
            // HTMX redirect on success
            (StatusCode::OK, [("HX-Redirect", "/boards.html")]).into_response()
        }
        Err(e) => {
            let msg = match &e {
                AppError::Unauthorized(m) => m.clone(),
                AppError::BadRequest(m) => m.clone(),
                AppError::Internal(m) => {
                    tracing::error!("HTMX login error: {}", m);
                    "An internal error occurred".into()
                }
                _ => "An error occurred".into(),
            };
            Html(format!("<div class=\"alert alert-error\">{}</div>", msg)).into_response()
        }
    }
}

const LOGIN_PAGE: &str = concat!(
    r##"<!DOCTYPE html>"##,
    r##"<html lang="en">"##,
    r##"<head>"##,
    r##"<meta charset="UTF-8">"##,
    r##"<meta name="viewport" content="width=device-width, initial-scale=1">"##,
    r##"<title>quest-board — Login</title>"##,
    r##"<link rel="stylesheet" href="/css/style.css">"##,
    r##"<script src="https://unpkg.com/htmx.org@2"></script>"##,
    r##"</head>"##,
    r##"<body>"##,
    r##"<div class="auth-page">"##,
    r##"  <div class="auth-box">"##,
    r##"    <h1>quest-board</h1>"##,
    r##"    <div id="alerts"></div>"##,
    r##"    <form hx-post="/login" hx-target="#alerts" hx-swap="innerHTML">"##,
    r##"      <div class="form-group">"##,
    r##"        <label>Username</label>"##,
    r##"        <input type="text" name="username" required autocomplete="username">"##,
    r##"      </div>"##,
    r##"      <div class="form-group">"##,
    r##"        <label>Password</label>"##,
    r##"        <input type="password" name="password" required autocomplete="current-password">"##,
    r##"      </div>"##,
    r##"      <button type="submit" class="btn btn-primary" style="width:100%">Sign In</button>"##,
    r##"    </form>"##,
    r##"    <p style="margin-top:16px;text-align:center;font-size:12px;color:var(--text2)">"##,
    r##"      Don't have an account? <a href="/register">Register</a>"##,
    r##"    </p>"##,
    r##"  </div>"##,
    r##"</div>"##,
    r##"</body>"##,
    r##"</html>"##,
);
