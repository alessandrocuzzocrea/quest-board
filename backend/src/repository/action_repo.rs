use crate::error::AppError;
use crate::models::action::Action;
use uuid::Uuid;

pub async fn record(
    pool: &sqlx::PgPool,
    card_id: &Uuid,
    user_id: Option<&Uuid>,
    action_type: &str,
    data: serde_json::Value,
) -> Result<(), AppError> {
    let now = chrono::Utc::now().to_rfc3339();
    let board_id: Option<Uuid> =
        sqlx::query_as::<_, (Uuid,)>("SELECT board_id FROM cards WHERE id = $1")
            .bind(card_id)
            .fetch_optional(pool)
            .await?
            .map(|r| r.0);

    sqlx::query(
        "INSERT INTO actions (card_id, board_id, user_id, action_type, data, created_at) VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(card_id)
    .bind(&board_id)
    .bind(user_id)
    .bind(action_type)
    .bind(data.to_string())
    .bind(&now)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn list_by_card(pool: &sqlx::PgPool, card_id: &Uuid) -> Result<Vec<Action>, AppError> {
    Ok(sqlx::query_as(
        "SELECT * FROM actions WHERE card_id = $1 ORDER BY created_at DESC LIMIT 50",
    )
    .bind(card_id)
    .fetch_all(pool)
    .await?)
}
