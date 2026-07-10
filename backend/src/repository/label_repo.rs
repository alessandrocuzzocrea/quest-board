use crate::error::AppError;
use crate::models::label::Label;

pub async fn list_by_board(pool: &sqlx::PgPool, board_id: &str) -> Result<Vec<Label>, AppError> {
    Ok(sqlx::query_as(
        "SELECT * FROM labels WHERE board_id = $1 ORDER BY position, name",
    )
    .bind(board_id)
    .fetch_all(pool)
    .await?)
}

pub async fn create(
    pool: &sqlx::PgPool,
    id: &str,
    board_id: &str,
    name: &str,
    color: &str,
    position: f64,
) -> Result<Label, AppError> {
    sqlx::query(
        "INSERT INTO labels (id, board_id, name, color, position) VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(id)
    .bind(board_id)
    .bind(name)
    .bind(color)
    .bind(position)
    .execute(pool)
    .await?;
    get_by_id(pool, id).await.transpose().unwrap()
}

pub async fn get_by_id(pool: &sqlx::PgPool, label_id: &str) -> Result<Option<Label>, AppError> {
    Ok(sqlx::query_as("SELECT * FROM labels WHERE id = $1")
        .bind(label_id)
        .fetch_optional(pool)
        .await?)
}

pub async fn update_name(pool: &sqlx::PgPool, label_id: &str, name: &str) -> Result<(), AppError> {
    sqlx::query("UPDATE labels SET name = $1, updated_at = NOW() WHERE id = $2")
        .bind(name).bind(label_id).execute(pool).await?;
    Ok(())
}

pub async fn update_color(pool: &sqlx::PgPool, label_id: &str, color: &str) -> Result<(), AppError> {
    sqlx::query("UPDATE labels SET color = $1, updated_at = NOW() WHERE id = $2")
        .bind(color).bind(label_id).execute(pool).await?;
    Ok(())
}

pub async fn update_position(pool: &sqlx::PgPool, label_id: &str, position: f64) -> Result<(), AppError> {
    sqlx::query("UPDATE labels SET position = $1, updated_at = NOW() WHERE id = $2")
        .bind(position).bind(label_id).execute(pool).await?;
    Ok(())
}

pub async fn delete(pool: &sqlx::PgPool, label_id: &str) -> Result<(), AppError> {
    sqlx::query("DELETE FROM labels WHERE id = $1")
        .bind(label_id).execute(pool).await?;
    Ok(())
}
