use crate::error::AppError;
use crate::models::attachment::Attachment;
use uuid::Uuid;

pub async fn list_by_card(pool: &sqlx::PgPool, card_id: &Uuid) -> Result<Vec<Attachment>, AppError> {
    Ok(sqlx::query_as(
        "SELECT * FROM attachments WHERE card_id = $1 ORDER BY created_at",
    )
    .bind(card_id)
    .fetch_all(pool)
    .await?)
}

pub async fn create_link(
    pool: &sqlx::PgPool,
    card_id: &Uuid,
    user_id: &Uuid,
    name: &str,
    url: &str,
) -> Result<Attachment, AppError> {
    let id: Uuid = sqlx::query_scalar(
        "INSERT INTO attachments (card_id, user_id, name, attachment_type, link_url) VALUES ($1, $2, $3, 'link', $4) RETURNING id",
    )
    .bind(card_id)
    .bind(user_id)
    .bind(name)
    .bind(url)
    .fetch_one(pool)
    .await?;
    get_by_id(pool, &id).await.transpose().unwrap()
}

pub async fn get_by_id(pool: &sqlx::PgPool, attachment_id: &Uuid) -> Result<Option<Attachment>, AppError> {
    Ok(sqlx::query_as("SELECT * FROM attachments WHERE id = $1")
        .bind(attachment_id)
        .fetch_optional(pool)
        .await?)
}

pub async fn delete(pool: &sqlx::PgPool, attachment_id: &Uuid) -> Result<(), AppError> {
    sqlx::query("DELETE FROM attachments WHERE id = $1")
        .bind(attachment_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn create_file(
    pool: &sqlx::PgPool,
    card_id: &Uuid,
    user_id: &Uuid,
    name: &str,
    file_path: &str,
    size: i64,
    mime_type: &str,
) -> Result<Attachment, AppError> {
    let id: Uuid = sqlx::query_scalar(
        "INSERT INTO attachments (card_id, user_id, name, attachment_type, file_path, size, mime_type) \
         VALUES ($1, $2, $3, 'file', $4, $5, $6) RETURNING id",
    )
    .bind(card_id)
    .bind(user_id)
    .bind(name)
    .bind(file_path)
    .bind(size)
    .bind(mime_type)
    .fetch_one(pool)
    .await?;
    get_by_id(pool, &id).await.transpose().unwrap()
}
