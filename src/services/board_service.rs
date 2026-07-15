use uuid::Uuid;

use crate::error::AppError;
use crate::models::board::Board;
use crate::models::list::ListWithCards;
use crate::models::user::UserResponse;
use crate::repository;

/// Business logic for boards, shared by HTML and JSON handlers.
pub struct BoardService {
    db: sqlx::PgPool,
}

impl BoardService {
    pub fn new(db: sqlx::PgPool) -> Self {
        Self { db }
    }

    pub async fn list_accessible(&self, user_id: &Uuid) -> Result<Vec<Board>, AppError> {
        repository::board_repo::list_accessible(&self.db, user_id).await
    }

    pub async fn create(&self, name: &str, created_by: &Uuid) -> Result<Board, AppError> {
        let board = repository::board_repo::create(&self.db, name, created_by).await?;
        repository::board_repo::add_member(&self.db, &board.id, created_by, "admin").await?;
        repository::list_repo::create_defaults(&self.db, &board.id).await?;
        Ok(board)
    }

    pub async fn get_full(&self, board_id: &Uuid) -> Result<(Board, Vec<ListWithCards>, Vec<UserResponse>), AppError> {
        repository::board_repo::get_full_board(&self.db, board_id).await
    }

    pub async fn get_by_slug(&self, slug: &str) -> Result<Option<Board>, AppError> {
        repository::board_repo::get_by_slug(&self.db, slug).await
    }

    pub async fn get_full_by_slug(&self, slug: &str) -> Result<(Board, Vec<ListWithCards>, Vec<UserResponse>), AppError> {
        repository::board_repo::get_full_board_by_slug(&self.db, slug).await
    }

    pub async fn update(&self, board_id: &Uuid, name: Option<&str>, position: Option<f64>) -> Result<Board, AppError> {
        if let Some(name) = name {
            repository::board_repo::update_name(&self.db, board_id, name).await?;
        }
        if let Some(position) = position {
            repository::board_repo::update_position(&self.db, board_id, position).await?;
        }
        repository::board_repo::get_by_id(&self.db, board_id)
            .await?
            .ok_or(AppError::NotFound("board not found".into()))
    }

    pub async fn delete(&self, board_id: &Uuid) -> Result<(), AppError> {
        repository::board_repo::delete(&self.db, board_id).await
    }
}
