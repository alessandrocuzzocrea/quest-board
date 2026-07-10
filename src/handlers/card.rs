use axum::extract::{Path, State};
use axum::routing::{get, post, put};
use axum::{Json, Router};
use std::sync::Arc;

use crate::error::AppError;
use crate::models::card::*;
use crate::AppState;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", post(create_card))
        .route("/{id}", get(get_card).put(update_card).delete(delete_card))
        .route("/{id}/move", put(move_card))
        .route("/{id}/members", post(add_member).delete(remove_member))
        .route("/{id}/labels", post(add_label).delete(remove_label))
        // Checklists (task lists)
        .route("/{id}/task-lists", post(create_task_list))
        .route("/{id}/task-lists/{tlid}", put(update_task_list).delete(delete_task_list))
        .route("/{id}/task-lists/{tlid}/tasks", post(create_task))
        .route("/{id}/task-lists/{tlid}/tasks/{tid}", put(update_task).delete(delete_task))
        .route("/{id}/comments", axum::routing::get(list_comments))
        .route("/{id}/actions", axum::routing::get(list_actions))
}

async fn require_user(session: &tower_sessions::Session) -> Result<String, AppError> {
    session
        .get::<String>("user_id")
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or(AppError::Unauthorized("not logged in".into()))
}

async fn record_action(
    db: &sqlx::PgPool,
    card_id: &str,
    user_id: Option<&str>,
    action_type: &str,
    data: serde_json::Value,
) -> Result<(), AppError> {
    let now = chrono::Utc::now().to_rfc3339();

    // Get board_id for the card
    let card: Option<crate::models::card::Card> =
        sqlx::query_as("SELECT * FROM cards WHERE id = $1")
            .bind(card_id)
            .fetch_optional(db)
            .await?;

    let board_id = card.as_ref().map(|c| c.board_id.clone());

    sqlx::query(
        "INSERT INTO actions (id, card_id, board_id, user_id, action_type, data, created_at) VALUES ($1, $2, $3, $4, $5, $6, $7)",
    )
    .bind(uuid::Uuid::new_v4().to_string())
    .bind(card_id)
    .bind(&board_id)
    .bind(user_id)
    .bind(action_type)
    .bind(data.to_string())
    .bind(&now)
    .execute(db)
    .await?;

    Ok(())
}

async fn create_card(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    Json(req): Json<CreateCardRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let user_id = require_user(&session).await?;
    let id = uuid::Uuid::new_v4().to_string();

    // Get list's board_id
    let list: crate::models::list::List =
        sqlx::query_as("SELECT * FROM lists WHERE id = $1")
            .bind(&req.list_id)
            .fetch_optional(&state.db)
            .await?
            .ok_or(AppError::NotFound("list not found".into()))?;

    sqlx::query(
        "INSERT INTO cards (id, board_id, list_id, name, description, created_by) VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(&id)
    .bind(&list.board_id)
    .bind(&req.list_id)
    .bind(&req.name)
    .bind(&req.description)
    .bind(&user_id)
    .execute(&state.db)
    .await?;

    record_action(
        &state.db,
        &id,
        Some(&user_id),
        "createCard",
        serde_json::json!({"card": {"name": &req.name}}),
    )
    .await?;

    let card: Card = sqlx::query_as("SELECT * FROM cards WHERE id = $1")
        .bind(&id)
        .fetch_one(&state.db)
        .await?;

    Ok(Json(serde_json::json!(card)))
}

async fn get_card(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    Path(card_id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let _user_id = require_user(&session).await?;

    let card: Card = sqlx::query_as("SELECT * FROM cards WHERE id = $1")
        .bind(&card_id)
        .fetch_optional(&state.db)
        .await?
        .ok_or(AppError::NotFound("card not found".into()))?;

    let members: Vec<crate::models::user::User> = sqlx::query_as(
        "SELECT u.* FROM users u JOIN card_members cm ON u.id = cm.user_id WHERE cm.card_id = $1",
    )
    .bind(&card_id)
    .fetch_all(&state.db)
    .await?;

    let labels: Vec<crate::models::label::Label> = sqlx::query_as(
        "SELECT l.* FROM labels l JOIN card_labels cl ON l.id = cl.label_id WHERE cl.card_id = $1",
    )
    .bind(&card_id)
    .fetch_all(&state.db)
    .await?;

    let comments: Vec<crate::models::comment::Comment> = sqlx::query_as(
        "SELECT * FROM comments WHERE card_id = $1 ORDER BY created_at",
    )
    .bind(&card_id)
    .fetch_all(&state.db)
    .await?;

    let checklists: Vec<crate::models::checklist::TaskList> = sqlx::query_as(
        "SELECT * FROM task_lists WHERE card_id = $1 ORDER BY position",
    )
    .bind(&card_id)
    .fetch_all(&state.db)
    .await?;

    let mut task_list_with_tasks: Vec<crate::models::checklist::TaskListWithTasks> = Vec::new();
    for tl in checklists {
        let tasks: Vec<crate::models::checklist::Task> = sqlx::query_as(
            "SELECT * FROM tasks WHERE task_list_id = $1 ORDER BY position",
        )
        .bind(&tl.id)
        .fetch_all(&state.db)
        .await?;
        task_list_with_tasks.push(crate::models::checklist::TaskListWithTasks {
            id: tl.id,
            card_id: tl.card_id,
            name: tl.name,
            position: tl.position,
            hide_completed: tl.hide_completed,
            tasks,
            created_at: tl.created_at,
            updated_at: tl.updated_at,
        });
    }

    let actions: Vec<crate::models::action::Action> = sqlx::query_as(
        "SELECT * FROM actions WHERE card_id = $1 ORDER BY created_at DESC LIMIT 50",
    )
    .bind(&card_id)
    .fetch_all(&state.db)
    .await?;

    Ok(Json(serde_json::json!({
        "card": card,
        "members": members.into_iter().map(crate::models::user::UserResponse::from).collect::<Vec<_>>(),
        "labels": labels,
        "comments": comments,
        "checklists": task_list_with_tasks,
        "actions": actions,
    })))
}

async fn update_card(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    Path(card_id): Path<String>,
    Json(req): Json<UpdateCardRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let user_id = require_user(&session).await?;

    let old_card: Card = sqlx::query_as("SELECT * FROM cards WHERE id = $1")
        .bind(&card_id)
        .fetch_optional(&state.db)
        .await?
        .ok_or(AppError::NotFound("card not found".into()))?;

    let mut list_changed = false;
    if let Some(ref list_id) = req.list_id {
        let list: crate::models::list::List = sqlx::query_as("SELECT * FROM lists WHERE id = $1")
            .bind(list_id)
            .fetch_optional(&state.db)
            .await?
            .ok_or(AppError::NotFound("target list not found".into()))?;

        sqlx::query(
            "UPDATE cards SET list_id = $1, board_id = $2, updated_at = NOW() WHERE id = $3",
        )
        .bind(list_id)
        .bind(&list.board_id)
        .bind(&card_id)
        .execute(&state.db)
        .await?;
        list_changed = true;
    }

    if let Some(ref name) = req.name {
        sqlx::query("UPDATE cards SET name = $1, updated_at = NOW() WHERE id = $2")
            .bind(name)
            .bind(&card_id)
            .execute(&state.db)
            .await?;
    }
    if let Some(ref description) = req.description {
        sqlx::query("UPDATE cards SET description = $1, updated_at = NOW() WHERE id = $2")
            .bind(description)
            .bind(&card_id)
            .execute(&state.db)
            .await?;
    }
    if let Some(position) = req.position {
        sqlx::query("UPDATE cards SET position = $1, updated_at = NOW() WHERE id = $2")
            .bind(position)
            .bind(&card_id)
            .execute(&state.db)
            .await?;
    }
    if let Some(ref due_date) = req.due_date {
        sqlx::query("UPDATE cards SET due_date = $1, updated_at = NOW() WHERE id = $2")
            .bind(due_date)
            .bind(&card_id)
            .execute(&state.db)
            .await?;
    }
    if let Some(is_due_completed) = req.is_due_completed {
        sqlx::query("UPDATE cards SET is_due_completed = $1, updated_at = NOW() WHERE id = $2")
            .bind(is_due_completed)
            .bind(&card_id)
            .execute(&state.db)
            .await?;
    }
    if let Some(is_closed) = req.is_closed {
        sqlx::query("UPDATE cards SET is_closed = $1, updated_at = NOW() WHERE id = $2")
            .bind(is_closed)
            .bind(&card_id)
            .execute(&state.db)
            .await?;
    }

    if list_changed {
        record_action(
            &state.db,
            &card_id,
            Some(&user_id),
            "moveCard",
            serde_json::json!({
                "fromList": {"id": old_card.list_id},
                "toList": {"id": req.list_id},
            }),
        )
        .await?;
    }

    let card: Card = sqlx::query_as("SELECT * FROM cards WHERE id = $1")
        .bind(&card_id)
        .fetch_optional(&state.db)
        .await?
        .ok_or(AppError::NotFound("card not found".into()))?;

    Ok(Json(serde_json::json!(card)))
}

async fn delete_card(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    Path(card_id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let _user_id = require_user(&session).await?;
    sqlx::query("DELETE FROM cards WHERE id = $1")
        .bind(&card_id)
        .execute(&state.db)
        .await?;
    Ok(Json(serde_json::json!({"ok": true})))
}

async fn move_card(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    Path(card_id): Path<String>,
    Json(req): Json<MoveCardRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let user_id = require_user(&session).await?;

    let old_card: Card = sqlx::query_as("SELECT * FROM cards WHERE id = $1")
        .bind(&card_id)
        .fetch_optional(&state.db)
        .await?
        .ok_or(AppError::NotFound("card not found".into()))?;

    let list: crate::models::list::List = sqlx::query_as("SELECT * FROM lists WHERE id = $1")
        .bind(&req.list_id)
        .fetch_optional(&state.db)
        .await?
        .ok_or(AppError::NotFound("target list not found".into()))?;

    sqlx::query(
        "UPDATE cards SET list_id = $1, board_id = $2, position = $3, updated_at = NOW() WHERE id = $4",
    )
    .bind(&req.list_id)
    .bind(&list.board_id)
    .bind(req.position)
    .bind(&card_id)
    .execute(&state.db)
    .await?;

    record_action(
        &state.db,
        &card_id,
        Some(&user_id),
        "moveCard",
        serde_json::json!({
            "fromList": {"id": old_card.list_id, "name": &old_card.name},
            "toList": {"id": req.list_id},
        }),
    )
    .await?;

    let card: Card = sqlx::query_as("SELECT * FROM cards WHERE id = $1")
        .bind(&card_id)
        .fetch_one(&state.db)
        .await?;

    Ok(Json(serde_json::json!(card)))
}

async fn add_member(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    Path(card_id): Path<String>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    let user_id = require_user(&session).await?;
    let member_id = req["user_id"].as_str().ok_or(AppError::BadRequest("user_id required".into()))?;

    sqlx::query(
        "INSERT OR IGNORE INTO card_members (id, card_id, user_id) VALUES ($1, $2, $3)",
    )
    .bind(uuid::Uuid::new_v4().to_string())
    .bind(&card_id)
    .bind(member_id)
    .execute(&state.db)
    .await?;

    record_action(
        &state.db,
        &card_id,
        Some(&user_id),
        "addMemberToCard",
        serde_json::json!({"userId": member_id}),
    )
    .await?;

    Ok(Json(serde_json::json!({"ok": true})))
}

async fn remove_member(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    Path(card_id): Path<String>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    let _user_id = require_user(&session).await?;
    let member_id = req["user_id"].as_str().ok_or(AppError::BadRequest("user_id required".into()))?;

    sqlx::query("DELETE FROM card_members WHERE card_id = $1 AND user_id = $2")
        .bind(&card_id)
        .bind(member_id)
        .execute(&state.db)
        .await?;

    Ok(Json(serde_json::json!({"ok": true})))
}

async fn add_label(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    Path(card_id): Path<String>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    let _user_id = require_user(&session).await?;
    let label_id = req["label_id"].as_str().ok_or(AppError::BadRequest("label_id required".into()))?;

    sqlx::query(
        "INSERT OR IGNORE INTO card_labels (id, card_id, label_id) VALUES ($1, $2, $3)",
    )
    .bind(uuid::Uuid::new_v4().to_string())
    .bind(&card_id)
    .bind(label_id)
    .execute(&state.db)
    .await?;

    Ok(Json(serde_json::json!({"ok": true})))
}

async fn remove_label(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    Path(card_id): Path<String>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    let _user_id = require_user(&session).await?;
    let label_id = req["label_id"].as_str().ok_or(AppError::BadRequest("label_id required".into()))?;

    sqlx::query("DELETE FROM card_labels WHERE card_id = $1 AND label_id = $2")
        .bind(&card_id)
        .bind(label_id)
        .execute(&state.db)
        .await?;

    Ok(Json(serde_json::json!({"ok": true})))
}

// Checklists (task lists)
async fn create_task_list(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    Path(card_id): Path<String>,
    Json(req): Json<crate::models::checklist::CreateTaskListRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let _user_id = require_user(&session).await?;
    let id = uuid::Uuid::new_v4().to_string();

    sqlx::query(
        "INSERT INTO task_lists (id, card_id, name, position) VALUES ($1, $2, $3, $4)",
    )
    .bind(&id)
    .bind(&req.card_id)
    .bind(&req.name)
    .bind(65536.0f64)
    .execute(&state.db)
    .await?;

    let tl: crate::models::checklist::TaskList = sqlx::query_as("SELECT * FROM task_lists WHERE id = $1")
        .bind(&id)
        .fetch_one(&state.db)
        .await?;

    Ok(Json(serde_json::json!(tl)))
}

async fn update_task_list(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    Path((_card_id, tlid)): Path<(String, String)>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    let _user_id = require_user(&session).await?;

    if let Some(name) = req["name"].as_str() {
        sqlx::query("UPDATE task_lists SET name = $1, updated_at = NOW() WHERE id = $2")
            .bind(name)
            .bind(&tlid)
            .execute(&state.db)
            .await?;
    }

    let tl: crate::models::checklist::TaskList = sqlx::query_as("SELECT * FROM task_lists WHERE id = $1")
        .bind(&tlid)
        .fetch_optional(&state.db)
        .await?
        .ok_or(AppError::NotFound("task list not found".into()))?;

    Ok(Json(serde_json::json!(tl)))
}

async fn delete_task_list(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    Path((_card_id, tlid)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>, AppError> {
    let _user_id = require_user(&session).await?;
    sqlx::query("DELETE FROM task_lists WHERE id = $1")
        .bind(&tlid)
        .execute(&state.db)
        .await?;
    Ok(Json(serde_json::json!({"ok": true})))
}

async fn create_task(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    Path((_card_id, tlid)): Path<(String, String)>,
    Json(req): Json<crate::models::checklist::CreateTaskRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let _user_id = require_user(&session).await?;
    let id = uuid::Uuid::new_v4().to_string();

    sqlx::query(
        "INSERT INTO tasks (id, task_list_id, name, position) VALUES ($1, $2, $3, $4)",
    )
    .bind(&id)
    .bind(&tlid)
    .bind(&req.name)
    .bind(65536.0f64)
    .execute(&state.db)
    .await?;

    let task: crate::models::checklist::Task = sqlx::query_as("SELECT * FROM tasks WHERE id = $1")
        .bind(&id)
        .fetch_one(&state.db)
        .await?;

    Ok(Json(serde_json::json!(task)))
}

async fn update_task(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    Path((_card_id, _tlid, tid)): Path<(String, String, String)>,
    Json(req): Json<crate::models::checklist::UpdateTaskRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let _user_id = require_user(&session).await?;

    if let Some(ref name) = req.name {
        sqlx::query("UPDATE tasks SET name = $1, updated_at = NOW() WHERE id = $2")
            .bind(name)
            .bind(&tid)
            .execute(&state.db)
            .await?;
    }
    if let Some(is_completed) = req.is_completed {
        sqlx::query("UPDATE tasks SET is_completed = $1, updated_at = NOW() WHERE id = $2")
            .bind(is_completed)
            .bind(&tid)
            .execute(&state.db)
            .await?;
    }
    if let Some(position) = req.position {
        sqlx::query("UPDATE tasks SET position = $1, updated_at = NOW() WHERE id = $2")
            .bind(position)
            .bind(&tid)
            .execute(&state.db)
            .await?;
    }
    if let Some(ref assignee_id) = req.assignee_id {
        sqlx::query("UPDATE tasks SET assignee_id = $1, updated_at = NOW() WHERE id = $2")
            .bind(assignee_id)
            .bind(&tid)
            .execute(&state.db)
            .await?;
    }

    let task: crate::models::checklist::Task = sqlx::query_as("SELECT * FROM tasks WHERE id = $1")
        .bind(&tid)
        .fetch_optional(&state.db)
        .await?
        .ok_or(AppError::NotFound("task not found".into()))?;

    Ok(Json(serde_json::json!(task)))
}

async fn delete_task(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    Path((_card_id, _tlid, tid)): Path<(String, String, String)>,
) -> Result<Json<serde_json::Value>, AppError> {
    let _user_id = require_user(&session).await?;
    sqlx::query("DELETE FROM tasks WHERE id = $1")
        .bind(&tid)
        .execute(&state.db)
        .await?;
    Ok(Json(serde_json::json!({"ok": true})))
}

async fn list_comments(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    Path(card_id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let _user_id = require_user(&session).await?;

    let comments: Vec<crate::models::comment::Comment> = sqlx::query_as(
        "SELECT * FROM comments WHERE card_id = $1 ORDER BY created_at",
    )
    .bind(&card_id)
    .fetch_all(&state.db)
    .await?;

    Ok(Json(serde_json::json!(comments)))
}

async fn list_actions(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    Path(card_id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let _user_id = require_user(&session).await?;

    let actions: Vec<crate::models::action::Action> = sqlx::query_as(
        "SELECT * FROM actions WHERE card_id = $1 ORDER BY created_at DESC LIMIT 50",
    )
    .bind(&card_id)
    .fetch_all(&state.db)
    .await?;

    Ok(Json(serde_json::json!(actions)))
}
