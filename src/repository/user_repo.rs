use crate::error::AppError;
use crate::models::user::{User, UserResponse};
use uuid::Uuid;

pub async fn find_by_username(pool: &sqlx::PgPool, username: &str) -> Result<Option<User>, AppError> {
    Ok(sqlx::query_as("SELECT * FROM users WHERE username = $1")
        .bind(username)
        .fetch_optional(pool)
        .await?)
}

pub async fn find_by_id(pool: &sqlx::PgPool, id: &Uuid) -> Result<Option<User>, AppError> {
    Ok(sqlx::query_as("SELECT * FROM users WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await?)
}

pub async fn create(
    pool: &sqlx::PgPool,
    username: &str,
    password_hash: &str,
) -> Result<User, AppError> {
    Ok(sqlx::query_as(
        "INSERT INTO users (username, password_hash) VALUES ($1, $2) RETURNING *",
    )
    .bind(username)
    .bind(password_hash)
    .fetch_one(pool)
    .await?)
}

pub async fn list_all(pool: &sqlx::PgPool) -> Result<Vec<UserResponse>, AppError> {
    let users: Vec<User> = sqlx::query_as("SELECT * FROM users ORDER BY username")
        .fetch_all(pool)
        .await?;
    Ok(users.into_iter().map(Into::into).collect())
}

pub async fn update_password(pool: &sqlx::PgPool, id: &Uuid, password_hash: &str) -> Result<(), AppError> {
    sqlx::query("UPDATE users SET password_hash = $1, updated_at = NOW() WHERE id = $2")
        .bind(password_hash)
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}
