use crate::error::AppError;
use crate::models::api_key::ApiKey;
use uuid::Uuid;
use time::OffsetDateTime;

pub async fn list_by_user(pool: &sqlx::PgPool, user_id: Uuid) -> Result<Vec<ApiKey>, AppError> {
    Ok(sqlx::query_as(
        "SELECT * FROM api_keys WHERE user_id = $1 ORDER BY created_at DESC",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?)
}

pub async fn create(
    pool: &sqlx::PgPool,
    user_id: Uuid,
    name: &str,
    token_hash: &str,
    prefix: &str,
    expires_at: Option<OffsetDateTime>,
) -> Result<ApiKey, AppError> {
    Ok(sqlx::query_as(
        "INSERT INTO api_keys (user_id, name, token_hash, prefix, expires_at) \
         VALUES ($1, $2, $3, $4, $5) RETURNING *",
    )
    .bind(user_id)
    .bind(name)
    .bind(token_hash)
    .bind(prefix)
    .bind(expires_at)
    .fetch_one(pool)
    .await?)
}

pub async fn delete(pool: &sqlx::PgPool, id: Uuid, user_id: Uuid) -> Result<(), AppError> {
    let result = sqlx::query(
        "UPDATE api_keys SET is_active = false WHERE id = $1 AND user_id = $2",
    )
    .bind(id)
    .bind(user_id)
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("API key not found".into()));
    }

    Ok(())
}

pub async fn find_by_token_hash(
    pool: &sqlx::PgPool,
    hash: &str,
) -> Result<Option<(Uuid,)>, AppError> {
    Ok(sqlx::query_as(
        "SELECT user_id FROM api_keys \
         WHERE token_hash = $1 AND is_active = true \
         AND (expires_at IS NULL OR expires_at > NOW())",
    )
    .bind(hash)
    .fetch_optional(pool)
    .await?)
}
