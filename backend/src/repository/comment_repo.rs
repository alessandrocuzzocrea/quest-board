use crate::error::AppError;
use crate::models::comment::{Comment, CommentWithUser};

pub async fn list_by_card(pool: &sqlx::PgPool, card_id: &str) -> Result<Vec<CommentWithUser>, AppError> {
    let comments: Vec<Comment> = sqlx::query_as(
        "SELECT * FROM comments WHERE card_id = $1 ORDER BY created_at",
    )
    .bind(card_id)
    .fetch_all(pool)
    .await?;

    let mut result: Vec<CommentWithUser> = Vec::new();
    for c in comments {
        let user: Option<crate::models::user::UserResponse> =
            if let Ok(Some(u)) = sqlx::query_as::<_, crate::models::user::User>(
                "SELECT * FROM users WHERE id = $1",
            )
            .bind(&c.user_id)
            .fetch_optional(pool)
            .await
            {
                Some(u.into())
            } else {
                None
            };
        result.push(CommentWithUser {
            id: c.id,
            card_id: c.card_id,
            user_id: c.user_id,
            user,
            text: c.text,
            created_at: c.created_at,
            updated_at: c.updated_at,
        });
    }
    Ok(result)
}

pub async fn create(
    pool: &sqlx::PgPool,
    id: &str,
    card_id: &str,
    user_id: &str,
    text: &str,
    now: &str,
) -> Result<Comment, AppError> {
    sqlx::query(
        "INSERT INTO comments (id, card_id, user_id, text, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(id)
    .bind(card_id)
    .bind(user_id)
    .bind(text)
    .bind(now)
    .bind(now)
    .execute(pool)
    .await?;

    sqlx::query("UPDATE cards SET updated_at = $1 WHERE id = $2")
        .bind(now)
        .bind(card_id)
        .execute(pool)
        .await?;

    get_by_id(pool, id).await.transpose().unwrap()
}

pub async fn get_by_id(pool: &sqlx::PgPool, comment_id: &str) -> Result<Option<Comment>, AppError> {
    Ok(sqlx::query_as("SELECT * FROM comments WHERE id = $1")
        .bind(comment_id)
        .fetch_optional(pool)
        .await?)
}

pub async fn update_text(pool: &sqlx::PgPool, comment_id: &str, text: &str) -> Result<Comment, AppError> {
    sqlx::query("UPDATE comments SET text = $1, updated_at = NOW() WHERE id = $2")
        .bind(text)
        .bind(comment_id)
        .execute(pool)
        .await?;
    get_by_id(pool, comment_id).await.transpose().unwrap()
}

pub async fn delete(pool: &sqlx::PgPool, comment_id: &str) -> Result<(), AppError> {
    sqlx::query("DELETE FROM comments WHERE id = $1")
        .bind(comment_id)
        .execute(pool)
        .await?;
    Ok(())
}
