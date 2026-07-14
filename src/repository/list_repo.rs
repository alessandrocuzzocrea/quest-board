use crate::error::AppError;
use crate::models::list::List;
use uuid::Uuid;

pub async fn create(pool: &sqlx::PgPool, board_id: &Uuid, name: &str, position: f64) -> Result<List, AppError> {
    let id: Uuid = sqlx::query_scalar(
        "INSERT INTO lists (board_id, name, position, list_type) VALUES ($1, $2, $3, 'active') RETURNING id",
    )
    .bind(board_id)
    .bind(name)
    .bind(position)
    .fetch_one(pool)
    .await?;
    get_by_id(pool, &id).await.transpose().unwrap()
}

pub async fn create_defaults(pool: &sqlx::PgPool, board_id: &Uuid) -> Result<(), AppError> {
    for (i, name) in ["To Do", "In Progress", "Done"].iter().enumerate() {
        create(pool, board_id, name, i as f64 * 65536.0).await?;
    }
    Ok(())
}

pub async fn get_by_id(pool: &sqlx::PgPool, list_id: &Uuid) -> Result<Option<List>, AppError> {
    Ok(sqlx::query_as("SELECT * FROM lists WHERE id = $1")
        .bind(list_id)
        .fetch_optional(pool)
        .await?)
}

pub async fn update_name(pool: &sqlx::PgPool, list_id: &Uuid, name: &str) -> Result<(), AppError> {
    sqlx::query("UPDATE lists SET name = $1, updated_at = NOW() WHERE id = $2")
        .bind(name)
        .bind(list_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn update_position(pool: &sqlx::PgPool, list_id: &Uuid, position: f64) -> Result<(), AppError> {
    sqlx::query("UPDATE lists SET position = $1, updated_at = NOW() WHERE id = $2")
        .bind(position)
        .bind(list_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn update_color(pool: &sqlx::PgPool, list_id: &Uuid, color: &str) -> Result<(), AppError> {
    sqlx::query("UPDATE lists SET color = $1, updated_at = NOW() WHERE id = $2")
        .bind(color)
        .bind(list_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn delete(pool: &sqlx::PgPool, list_id: &Uuid) -> Result<(), AppError> {
    sqlx::query("DELETE FROM lists WHERE id = $1")
        .bind(list_id)
        .execute(pool)
        .await?;
    Ok(())
}
