use uuid::Uuid;

use crate::error::AppError;
use crate::repository;

/// Business logic for favorites, shared by HTML and JSON handlers.
pub struct FavoriteService {
    db: sqlx::PgPool,
}

impl FavoriteService {
    pub fn new(db: sqlx::PgPool) -> Self {
        Self { db }
    }

    pub async fn list_by_user(&self, user_id: &Uuid) -> Result<serde_json::Value, AppError> {
        repository::favorite_repo::list_by_user(&self.db, user_id).await
    }

    pub async fn create(&self, user_id: &Uuid, board_id: Option<&Uuid>, card_id: Option<&Uuid>) -> Result<(), AppError> {
        repository::favorite_repo::create(&self.db, user_id, board_id, card_id).await
    }

    pub async fn delete(&self, fav_id: &Uuid) -> Result<(), AppError> {
        repository::favorite_repo::delete(&self.db, fav_id).await
    }
}
