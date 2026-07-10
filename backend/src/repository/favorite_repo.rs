use crate::error::AppError;
use crate::models::favorite::Favorite;
use serde_json::json;

pub async fn list_by_user(pool: &sqlx::PgPool, user_id: &str) -> Result<serde_json::Value, AppError> {
    let favorites: Vec<Favorite> = sqlx::query_as(
        "SELECT * FROM favorites WHERE user_id = $1 ORDER BY created_at",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    let mut boards: Vec<serde_json::Value> = Vec::new();
    let mut cards: Vec<serde_json::Value> = Vec::new();

    for f in favorites {
        if let Some(ref board_id) = f.board_id {
            let board: Option<crate::models::board::Board> =
                sqlx::query_as("SELECT * FROM boards WHERE id = $1")
                    .bind(board_id)
                    .fetch_optional(pool)
                    .await?;
            if let Some(b) = board {
                boards.push(json!({"board_id": b.id, "name": b.name}));
            }
        }
        if let Some(ref card_id_val) = f.card_id {
            let card: Option<(String, String)> =
                sqlx::query_as("SELECT id, name FROM cards WHERE id = $1")
                    .bind(card_id_val)
                    .fetch_optional(pool)
                    .await?;
            if let Some((cid, cname)) = card {
                cards.push(json!({"card_id": cid, "name": cname}));
            }
        }
    }

    Ok(json!({"boards": boards, "cards": cards}))
}

pub async fn create(
    pool: &sqlx::PgPool,
    user_id: &str,
    board_id: Option<&str>,
    card_id: Option<&str>,
) -> Result<(), AppError> {
    sqlx::query(
        "INSERT INTO favorites (id, user_id, board_id, card_id) VALUES ($1, $2, $3, $4)",
    )
    .bind(uuid::Uuid::new_v4().to_string())
    .bind(user_id)
    .bind(board_id)
    .bind(card_id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn delete(pool: &sqlx::PgPool, fav_id: &str) -> Result<(), AppError> {
    sqlx::query("DELETE FROM favorites WHERE id = $1")
        .bind(fav_id)
        .execute(pool)
        .await?;
    Ok(())
}
