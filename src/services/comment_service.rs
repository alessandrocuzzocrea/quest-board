use tokio::sync::broadcast;
use uuid::Uuid;

use crate::error::AppError;
use crate::events::{emit_simple, SseEvent};
use crate::models::comment::{Comment, CommentWithUser, CreateCommentRequest, UpdateCommentRequest};
use crate::repository;

/// Business logic for comments, shared by HTML and JSON handlers.
pub struct CommentService {
    db: sqlx::PgPool,
    event_tx: broadcast::Sender<SseEvent>,
}

impl CommentService {
    pub fn new(db: sqlx::PgPool, event_tx: broadcast::Sender<SseEvent>) -> Self {
        Self { db, event_tx }
    }

    pub async fn list_by_card(&self, card_id: &Uuid) -> Result<Vec<CommentWithUser>, AppError> {
        repository::comment_repo::list_by_card(&self.db, card_id).await
    }

    pub async fn create(&self, req: &CreateCommentRequest, user_id: &Uuid) -> Result<Comment, AppError> {
        let now = chrono::Utc::now().to_rfc3339();

        let comment = repository::comment_repo::create(&self.db, &req.card_id, user_id, &req.text, &now).await?;

        repository::action_repo::record(
            &self.db, &req.card_id, Some(user_id), "commentCard",
            serde_json::json!({"comment": {"text": &req.text}}),
        ).await?;

        emit_simple(&self.event_tx, "comment_created", "",
            Some(&req.card_id.to_string()), None, &user_id.to_string());

        Ok(comment)
    }

    pub async fn update(&self, comment_id: &Uuid, req: &UpdateCommentRequest, user_id: &Uuid) -> Result<Comment, AppError> {
        let comment = repository::comment_repo::update_text(&self.db, comment_id, &req.text).await?;
        emit_simple(&self.event_tx, "comment_updated", "",
            Some(&comment.card_id.to_string()), None, &user_id.to_string());
        Ok(comment)
    }

    pub async fn delete(&self, comment_id: &Uuid, user_id: &Uuid) -> Result<(), AppError> {
        let comment = repository::comment_repo::get_by_id(&self.db, comment_id).await?;
        let card_id = comment.map(|c| c.card_id.to_string()).unwrap_or_default();
        repository::comment_repo::delete(&self.db, comment_id).await?;
        emit_simple(&self.event_tx, "comment_deleted", "",
            Some(&card_id), None, &user_id.to_string());
        Ok(())
    }
}
