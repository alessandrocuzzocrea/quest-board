use tokio::sync::broadcast;
use uuid::Uuid;

use crate::error::AppError;
use crate::events::{emit_simple, SseEvent};
use crate::models::card::Card;
use crate::models::list::{CreateListRequest, List, UpdateListRequest};
use crate::repository;

/// Business logic for lists, shared by HTML and JSON handlers.
pub struct ListService {
    db: sqlx::PgPool,
    event_tx: broadcast::Sender<SseEvent>,
}

impl ListService {
    pub fn new(db: sqlx::PgPool, event_tx: broadcast::Sender<SseEvent>) -> Self {
        Self { db, event_tx }
    }

    pub async fn create(&self, req: &CreateListRequest, user_id: &Uuid) -> Result<List, AppError> {
        let list = repository::list_repo::create(
            &self.db, &req.board_id,
            req.name.as_deref().unwrap_or("New List"),
            65536.0,
        ).await?;
        emit_simple(&self.event_tx, "list_created", &req.board_id.to_string(),
            None, Some(&list.id.to_string()), &user_id.to_string());
        Ok(list)
    }

    pub async fn get_with_cards(&self, list_id: &Uuid) -> Result<(List, Vec<Card>), AppError> {
        let list = repository::list_repo::get_by_id(&self.db, list_id)
            .await?
            .ok_or(AppError::NotFound("list not found".into()))?;

        let cards: Vec<Card> = sqlx::query_as(
            "SELECT * FROM cards WHERE list_id = $1 ORDER BY position, created_at",
        )
        .bind(list_id)
        .fetch_all(&self.db)
        .await?;

        Ok((list, cards))
    }

    pub async fn update(&self, list_id: &Uuid, req: &UpdateListRequest, user_id: &Uuid) -> Result<List, AppError> {
        if let Some(name) = &req.name {
            repository::list_repo::update_name(&self.db, list_id, name).await?;
        }
        if let Some(position) = req.position {
            repository::list_repo::update_position(&self.db, list_id, position).await?;
        }
        if let Some(color) = &req.color {
            repository::list_repo::update_color(&self.db, list_id, color).await?;
        }
        let list = repository::list_repo::get_by_id(&self.db, list_id)
            .await?
            .ok_or(AppError::NotFound("list not found".into()))?;
        emit_simple(&self.event_tx, "list_updated", &list.board_id.to_string(),
            None, Some(&list.id.to_string()), &user_id.to_string());
        Ok(list)
    }

    pub async fn delete(&self, list_id: &Uuid, user_id: &Uuid) -> Result<(), AppError> {
        // Get board_id before deletion for event emission
        let board_id = repository::list_repo::get_by_id(&self.db, list_id)
            .await
            .ok()
            .flatten()
            .map(|l| l.board_id.to_string())
            .unwrap_or_default();

        repository::list_repo::delete(&self.db, list_id).await?;
        emit_simple(&self.event_tx, "list_deleted", &board_id,
            None, Some(&list_id.to_string()), &user_id.to_string());
        Ok(())
    }
}
