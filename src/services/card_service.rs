use tokio::sync::broadcast;
use uuid::Uuid;

use crate::error::AppError;
use crate::events::{emit_simple, SseEvent};
use crate::models::card::{
    Card, CreateCardRequest, MoveCardRequest, UpdateCardRequest,
};
use crate::models::checklist::{
    CreateTaskListRequest, CreateTaskRequest, Task, TaskList, UpdateTaskRequest,
};
use crate::models::comment::CommentWithUser;
use crate::models::action::Action;
use crate::repository;

/// Business logic for cards, shared by HTML and JSON handlers.
pub struct CardService {
    db: sqlx::PgPool,
    event_tx: broadcast::Sender<SseEvent>,
}

impl CardService {
    pub fn new(db: sqlx::PgPool, event_tx: broadcast::Sender<SseEvent>) -> Self {
        Self { db, event_tx }
    }

    // ── Card CRUD ─────────────────────────────────────────────

    pub async fn create(&self, req: &CreateCardRequest, user_id: &Uuid) -> Result<Card, AppError> {
        let list = repository::list_repo::get_by_id(&self.db, &req.list_id)
            .await?
            .ok_or(AppError::NotFound("list not found".into()))?;

        let card = repository::card_repo::create(
            &self.db, &list.board_id, &req.list_id, &req.name, &req.description, user_id,
        ).await?;

        repository::action_repo::record(
            &self.db, &card.id, Some(user_id), "createCard",
            serde_json::json!({"card": {"name": &req.name}}),
        ).await?;

        emit_simple(&self.event_tx, "card_created", &list.board_id.to_string(),
            Some(&card.id.to_string()), Some(&req.list_id.to_string()), &user_id.to_string());

        Ok(card)
    }

    pub async fn get_with_relations(&self, card_id: &Uuid) -> Result<serde_json::Value, AppError> {
        let card = repository::card_repo::get_by_id(&self.db, card_id)
            .await?
            .ok_or(AppError::NotFound("card not found".into()))?;

        let members = repository::card_repo::list_members(&self.db, card_id).await?;
        let labels = repository::card_repo::list_labels(&self.db, card_id).await?;
        let comments = repository::comment_repo::list_by_card(&self.db, card_id).await?;
        let checklists = repository::checklist_repo::list_by_card(&self.db, card_id).await?;
        let actions = repository::action_repo::list_by_card(&self.db, card_id).await?;

        let card_json = serde_json::to_value(&card).unwrap();
        let mut merged = card_json.as_object().unwrap().clone();
        merged.insert("members".into(), serde_json::to_value(&members).unwrap());
        merged.insert("labels".into(), serde_json::to_value(&labels).unwrap());
        merged.insert("comments".into(), serde_json::to_value(&comments).unwrap());
        merged.insert("checklists".into(), serde_json::to_value(&checklists).unwrap());
        merged.insert("actions".into(), serde_json::to_value(&actions).unwrap());
        Ok(serde_json::Value::Object(merged))
    }

    pub async fn update(&self, card_id: &Uuid, req: &UpdateCardRequest, user_id: &Uuid) -> Result<Card, AppError> {
        let card = repository::card_repo::update_card(&self.db, card_id, req).await?;
        emit_simple(&self.event_tx, "card_updated", &card.board_id.to_string(),
            Some(&card.id.to_string()), Some(&card.list_id.to_string()), &user_id.to_string());
        Ok(card)
    }

    pub async fn delete(&self, card_id: &Uuid, user_id: &Uuid) -> Result<(), AppError> {
        repository::card_repo::delete(&self.db, card_id).await?;
        emit_simple(&self.event_tx, "card_deleted", "",
            Some(&card_id.to_string()), None, &user_id.to_string());
        Ok(())
    }

    pub async fn move_card(&self, card_id: &Uuid, req: &MoveCardRequest, user_id: &Uuid) -> Result<Card, AppError> {
        let old = repository::card_repo::get_by_id(&self.db, card_id)
            .await?
            .ok_or(AppError::NotFound("card not found".into()))?;

        let card = repository::card_repo::move_card(&self.db, card_id, &req.list_id, req.position).await?;

        repository::action_repo::record(
            &self.db, card_id, Some(user_id), "moveCard",
            serde_json::json!({"fromList": {"id": old.list_id}, "toList": {"id": req.list_id}}),
        ).await?;

        emit_simple(&self.event_tx, "card_moved", &old.board_id.to_string(),
            Some(&card.id.to_string()), Some(&req.list_id.to_string()), &user_id.to_string());

        Ok(card)
    }

    // ── Members ────────────────────────────────────────────────

    pub async fn add_member(&self, card_id: &Uuid, member_id: &Uuid, user_id: &Uuid) -> Result<(), AppError> {
        repository::card_repo::add_member(&self.db, card_id, member_id).await?;
        repository::action_repo::record(
            &self.db, card_id, Some(user_id), "addMemberToCard",
            serde_json::json!({"userId": member_id.to_string()}),
        ).await?;
        Ok(())
    }

    pub async fn remove_member(&self, card_id: &Uuid, member_id: &Uuid) -> Result<(), AppError> {
        repository::card_repo::remove_member(&self.db, card_id, member_id).await
    }

    // ── Labels ─────────────────────────────────────────────────

    pub async fn add_label(&self, card_id: &Uuid, label_id: &Uuid) -> Result<(), AppError> {
        repository::card_repo::add_label(&self.db, card_id, label_id).await
    }

    pub async fn remove_label(&self, card_id: &Uuid, label_id: &Uuid) -> Result<(), AppError> {
        repository::card_repo::remove_label(&self.db, card_id, label_id).await
    }

    // ── Checklists ─────────────────────────────────────────────

    pub async fn create_task_list(&self, req: &CreateTaskListRequest) -> Result<TaskList, AppError> {
        repository::checklist_repo::create_task_list(&self.db, &req.card_id, &req.name, 65536.0).await
    }

    pub async fn update_task_list(&self, tlid: &Uuid, name: &str) -> Result<TaskList, AppError> {
        repository::checklist_repo::update_task_list_name(&self.db, tlid, name).await?;
        repository::checklist_repo::task_list_by_id(&self.db, tlid)
            .await?
            .ok_or(AppError::NotFound("task list not found".into()))
    }

    pub async fn delete_task_list(&self, tlid: &Uuid) -> Result<(), AppError> {
        repository::checklist_repo::delete_task_list(&self.db, tlid).await
    }

    pub async fn create_task(&self, tlid: &Uuid, req: &CreateTaskRequest) -> Result<Task, AppError> {
        repository::checklist_repo::create_task(&self.db, tlid, &req.name, 65536.0).await
    }

    pub async fn update_task(&self, tid: &Uuid, req: &UpdateTaskRequest) -> Result<Task, AppError> {
        repository::checklist_repo::update_task(
            &self.db, tid,
            req.name.as_deref(),
            req.is_completed,
            req.position,
            req.assignee_id.as_ref(),
        ).await
    }

    pub async fn delete_task(&self, tid: &Uuid) -> Result<(), AppError> {
        repository::checklist_repo::delete_task(&self.db, tid).await
    }

    // ── Related data ───────────────────────────────────────────

    pub async fn list_comments(&self, card_id: &Uuid) -> Result<Vec<CommentWithUser>, AppError> {
        repository::comment_repo::list_by_card(&self.db, card_id).await
    }

    pub async fn list_actions(&self, card_id: &Uuid) -> Result<Vec<Action>, AppError> {
        repository::action_repo::list_by_card(&self.db, card_id).await
    }
}
