use crate::error::AppError;
use crate::models::attachment::Attachment;

pub async fn list_by_card(pool: &sqlx::PgPool, card_id: &str) -> Result<Vec<Attachment>, AppError> {
    Ok(sqlx::query_as(
        "SELECT * FROM attachments WHERE card_id = $1 ORDER BY created_at",
    )
    .bind(card_id)
    .fetch_all(pool)
    .await?)
}

pub async fn create_link(
    pool: &sqlx::PgPool,
    id: &str,
    card_id: &str,
    user_id: &str,
    name: &str,
    url: &str,
) -> Result<Attachment, AppError> {
    sqlx::query(
        "INSERT INTO attachments (id, card_id, user_id, name, attachment_type, link_url) VALUES ($1, $2, $3, $4, 'link', $5)",
    )
    .bind(id)
    .bind(card_id)
    .bind(user_id)
    .bind(name)
    .bind(url)
    .execute(pool)
    .await?;
    get_by_id(pool, id).await.transpose().unwrap()
}

pub async fn get_by_id(pool: &sqlx::PgPool, attachment_id: &str) -> Result<Option<Attachment>, AppError> {
    Ok(sqlx::query_as("SELECT * FROM attachments WHERE id = $1")
        .bind(attachment_id)
        .fetch_optional(pool)
        .await?)
}

pub async fn delete(pool: &sqlx::PgPool, attachment_id: &str) -> Result<(), AppError> {
    sqlx::query("DELETE FROM attachments WHERE id = $1")
        .bind(attachment_id)
        .execute(pool)
        .await?;
    Ok(())
}
