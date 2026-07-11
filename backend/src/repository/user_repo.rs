use crate::error::AppError;
use crate::models::user::{User, UserResponse};
use uuid::Uuid;

pub async fn find_by_email(pool: &sqlx::PgPool, email: &str) -> Result<Option<User>, AppError> {
    Ok(sqlx::query_as("SELECT * FROM users WHERE email = $1")
        .bind(email)
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
    email: &str,
    password_hash: &str,
    name: &str,
) -> Result<User, AppError> {
    Ok(sqlx::query_as(
        "INSERT INTO users (email, password_hash, name) VALUES ($1, $2, $3) RETURNING *",
    )
    .bind(email)
    .bind(password_hash)
    .bind(name)
    .fetch_one(pool)
    .await?)
}

pub async fn list_all(pool: &sqlx::PgPool) -> Result<Vec<UserResponse>, AppError> {
    let users: Vec<User> = sqlx::query_as("SELECT * FROM users ORDER BY name")
        .fetch_all(pool)
        .await?;
    Ok(users.into_iter().map(Into::into).collect())
}
