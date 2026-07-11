use crate::error::AppError;
use crate::models::card::{Card, UpdateCardRequest};
use crate::models::list::List;
use crate::models::user::UserResponse;
use uuid::Uuid;

pub async fn create(pool: &sqlx::PgPool, board_id: &Uuid, list_id: &Uuid, name: &str, description: &Option<String>, created_by: &Uuid) -> Result<Card, AppError> {
    Ok(sqlx::query_as(
        "INSERT INTO cards (board_id, list_id, name, description, created_by) VALUES ($1, $2, $3, $4, $5) RETURNING *",
    )
    .bind(board_id)
    .bind(list_id)
    .bind(name)
    .bind(description)
    .bind(created_by)
    .fetch_one(pool)
    .await?)
}

pub async fn get_by_id(pool: &sqlx::PgPool, card_id: &Uuid) -> Result<Option<Card>, AppError> {
    Ok(sqlx::query_as("SELECT * FROM cards WHERE id = $1")
        .bind(card_id)
        .fetch_optional(pool)
        .await?)
}

pub async fn get_list_id(pool: &sqlx::PgPool, list_id: &Uuid) -> Result<Option<List>, AppError> {
    Ok(sqlx::query_as("SELECT * FROM lists WHERE id = $1")
        .bind(list_id)
        .fetch_optional(pool)
        .await?)
}

pub async fn update_card(pool: &sqlx::PgPool, card_id: &Uuid, req: &UpdateCardRequest) -> Result<Card, AppError> {
    if let Some(list_id) = &req.list_id {
        let list = get_list_id(pool, list_id).await?.ok_or(AppError::NotFound("target list not found".into()))?;
        sqlx::query("UPDATE cards SET list_id = $1, board_id = $2, updated_at = NOW() WHERE id = $3")
            .bind(list_id)
            .bind(&list.board_id)
            .bind(card_id)
            .execute(pool)
            .await?;
    }
    if let Some(name) = &req.name {
        sqlx::query("UPDATE cards SET name = $1, updated_at = NOW() WHERE id = $2")
            .bind(name).bind(card_id).execute(pool).await?;
    }
    if let Some(description) = &req.description {
        sqlx::query("UPDATE cards SET description = $1, updated_at = NOW() WHERE id = $2")
            .bind(description).bind(card_id).execute(pool).await?;
    }
    if let Some(position) = req.position {
        sqlx::query("UPDATE cards SET position = $1, updated_at = NOW() WHERE id = $2")
            .bind(position).bind(card_id).execute(pool).await?;
    }
    if let Some(due_date) = &req.due_date {
        sqlx::query("UPDATE cards SET due_date = $1, updated_at = NOW() WHERE id = $2")
            .bind(due_date).bind(card_id).execute(pool).await?;
    }
    if let Some(is_due_completed) = req.is_due_completed {
        sqlx::query("UPDATE cards SET is_due_completed = $1, updated_at = NOW() WHERE id = $2")
            .bind(is_due_completed).bind(card_id).execute(pool).await?;
    }
    if let Some(is_closed) = req.is_closed {
        sqlx::query("UPDATE cards SET is_closed = $1, updated_at = NOW() WHERE id = $2")
            .bind(is_closed).bind(card_id).execute(pool).await?;
    }
    get_by_id(pool, card_id).await.transpose().unwrap()
}

pub async fn move_card(pool: &sqlx::PgPool, card_id: &Uuid, list_id: &Uuid, position: f64) -> Result<Card, AppError> {
    let list = get_list_id(pool, list_id).await?.ok_or(AppError::NotFound("target list not found".into()))?;
    sqlx::query(
        "UPDATE cards SET list_id = $1, board_id = $2, position = $3, updated_at = NOW() WHERE id = $4",
    )
    .bind(list_id)
    .bind(&list.board_id)
    .bind(position)
    .bind(card_id)
    .execute(pool)
    .await?;
    get_by_id(pool, card_id).await.transpose().unwrap()
}

pub async fn delete(pool: &sqlx::PgPool, card_id: &Uuid) -> Result<(), AppError> {
    sqlx::query("DELETE FROM cards WHERE id = $1")
        .bind(card_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn add_member(pool: &sqlx::PgPool, card_id: &Uuid, user_id: &Uuid) -> Result<(), AppError> {
    sqlx::query("INSERT INTO card_members (card_id, user_id) VALUES ($1, $2) ON CONFLICT DO NOTHING")
        .bind(card_id)
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn remove_member(pool: &sqlx::PgPool, card_id: &Uuid, user_id: &Uuid) -> Result<(), AppError> {
    sqlx::query("DELETE FROM card_members WHERE card_id = $1 AND user_id = $2")
        .bind(card_id)
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn add_label(pool: &sqlx::PgPool, card_id: &Uuid, label_id: &Uuid) -> Result<(), AppError> {
    sqlx::query("INSERT INTO card_labels (card_id, label_id) VALUES ($1, $2)")
        .bind(card_id)
        .bind(label_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn remove_label(pool: &sqlx::PgPool, card_id: &Uuid, label_id: &Uuid) -> Result<(), AppError> {
    sqlx::query("DELETE FROM card_labels WHERE card_id = $1 AND label_id = $2")
        .bind(card_id)
        .bind(label_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn list_members(pool: &sqlx::PgPool, card_id: &Uuid) -> Result<Vec<UserResponse>, AppError> {
    let users: Vec<crate::models::user::User> = sqlx::query_as(
        "SELECT u.* FROM users u JOIN card_members cm ON u.id = cm.user_id WHERE cm.card_id = $1",
    )
    .bind(card_id)
    .fetch_all(pool)
    .await?;
    Ok(users.into_iter().map(Into::into).collect())
}

pub async fn list_labels(pool: &sqlx::PgPool, card_id: &Uuid) -> Result<Vec<crate::models::label::Label>, AppError> {
    Ok(sqlx::query_as(
        "SELECT l.* FROM labels l JOIN card_labels cl ON l.id = cl.label_id WHERE cl.card_id = $1",
    )
    .bind(card_id)
    .fetch_all(pool)
    .await?)
}
