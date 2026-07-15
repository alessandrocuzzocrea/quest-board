use uuid::Uuid;

use crate::error::AppError;
use crate::models::label::{CreateLabelRequest, Label, UpdateLabelRequest};
use crate::repository;

/// Business logic for labels, shared by HTML and JSON handlers.
pub struct LabelService {
    db: sqlx::PgPool,
}

impl LabelService {
    pub fn new(db: sqlx::PgPool) -> Self {
        Self { db }
    }

    pub async fn list_by_board(&self, board_id: &Uuid) -> Result<Vec<Label>, AppError> {
        repository::label_repo::list_by_board(&self.db, board_id).await
    }

    pub async fn create(&self, req: &CreateLabelRequest) -> Result<Label, AppError> {
        let color = req.color.as_deref().unwrap_or("#0079bf");
        repository::label_repo::create(&self.db, &req.board_id, &req.name, color, 65536.0).await
    }

    pub async fn update(&self, label_id: &Uuid, req: &UpdateLabelRequest) -> Result<Label, AppError> {
        if let Some(name) = &req.name {
            repository::label_repo::update_name(&self.db, label_id, name).await?;
        }
        if let Some(color) = &req.color {
            repository::label_repo::update_color(&self.db, label_id, color).await?;
        }
        if let Some(position) = req.position {
            repository::label_repo::update_position(&self.db, label_id, position).await?;
        }
        repository::label_repo::get_by_id(&self.db, label_id)
            .await?
            .ok_or(AppError::NotFound("label not found".into()))
    }

    pub async fn delete(&self, label_id: &Uuid) -> Result<(), AppError> {
        repository::label_repo::delete(&self.db, label_id).await
    }
}
