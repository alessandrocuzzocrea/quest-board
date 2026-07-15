use uuid::Uuid;

use crate::error::AppError;
use crate::models::board::Board;

/// Business logic for search, shared by HTML and JSON handlers.
pub struct SearchService {
    db: sqlx::PgPool,
}

impl SearchService {
    pub fn new(db: sqlx::PgPool) -> Self {
        Self { db }
    }

    pub async fn search_boards(&self, user_id: &Uuid, pattern: &str) -> Result<Vec<Board>, AppError> {
        let pattern = format!("%{}%", pattern);
        Ok(sqlx::query_as(
            "SELECT DISTINCT b.* FROM boards b
             LEFT JOIN board_members bm ON b.id = bm.board_id AND bm.user_id = $1
             WHERE b.name LIKE $2 AND (b.created_by = $3 OR bm.user_id = $4)
             ORDER BY b.name
             LIMIT 10",
        )
        .bind(user_id)
        .bind(&pattern)
        .bind(user_id)
        .bind(user_id)
        .fetch_all(&self.db)
        .await?)
    }

    pub async fn search_cards(&self, user_id: &Uuid, pattern: &str) -> Result<Vec<serde_json::Value>, AppError> {
        let pattern = format!("%{}%", pattern);
        let cards: Vec<(Uuid, String, Uuid, String, f64)> = sqlx::query_as(
            "SELECT c.id, c.name, c.board_id, l.name as list_name, c.position
             FROM cards c
             JOIN lists l ON c.list_id = l.id
             JOIN boards b ON c.board_id = b.id
             LEFT JOIN board_members bm ON b.id = bm.board_id AND bm.user_id = $1
             WHERE (c.name LIKE $2 OR c.description LIKE $3)
               AND (b.created_by = $4 OR bm.user_id = $5)
             ORDER BY c.position
             LIMIT 20",
        )
        .bind(user_id)
        .bind(&pattern)
        .bind(&pattern)
        .bind(user_id)
        .bind(user_id)
        .fetch_all(&self.db)
        .await?;

        Ok(cards.into_iter().map(|(id, name, board_id, list_name, _pos)| {
            serde_json::json!({"id": id, "name": name, "board_id": board_id, "list_name": list_name})
        }).collect())
    }
}
