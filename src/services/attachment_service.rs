use uuid::Uuid;

use crate::error::AppError;
use crate::models::attachment::Attachment;
use crate::repository;

/// Business logic for attachments, shared by HTML and JSON handlers.
pub struct AttachmentService {
    db: sqlx::PgPool,
}

impl AttachmentService {
    pub fn new(db: sqlx::PgPool) -> Self {
        Self { db }
    }

    pub async fn list_by_card(&self, card_id: &Uuid) -> Result<Vec<Attachment>, AppError> {
        repository::attachment_repo::list_by_card(&self.db, card_id).await
    }

    pub async fn create_link(&self, card_id: &Uuid, user_id: &Uuid, name: &str, url: &str) -> Result<Attachment, AppError> {
        repository::attachment_repo::create_link(&self.db, card_id, user_id, name, url).await
    }

    pub async fn delete(&self, attachment_id: &Uuid) -> Result<(), AppError> {
        repository::attachment_repo::delete(&self.db, attachment_id).await
    }

    pub async fn create_file(
        &self,
        card_id: &Uuid,
        user_id: &Uuid,
        name: &str,
        file_path: &str,
        size: i64,
        mime_type: &str,
    ) -> Result<Attachment, AppError> {
        repository::attachment_repo::create_file(&self.db, card_id, user_id, name, file_path, size, mime_type).await
    }
}
