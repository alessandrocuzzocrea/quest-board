use uuid::Uuid;

use crate::error::AppError;
use crate::models::board::Board;
use crate::repository;

/// Business logic for boards, shared by HTML and JSON handlers.
pub struct BoardService {
    db: sqlx::PgPool,
}

impl BoardService {
    pub fn new(db: sqlx::PgPool) -> Self {
        Self { db }
    }

    /// List boards accessible to a user.
    pub async fn list_accessible(&self, user_id: &Uuid) -> Result<Vec<Board>, AppError> {
        repository::board_repo::list_accessible(&self.db, user_id).await
    }
}
