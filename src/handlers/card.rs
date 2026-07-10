use axum::extract::{Path, State};
use axum::routing::{get, post, put};
use axum::{Json, Router};
use std::sync::Arc;

use crate::error::AppError;
use crate::models::card::*;
use crate::models::checklist::*;
use crate::repository;
use crate::AppState;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", post(create_card))
        .route("/{id}", get(get_card).put(update_card).delete(delete_card))
        .route("/{id}/move", put(move_card))
        .route("/{id}/members", post(add_member).delete(remove_member))
        .route("/{id}/labels", post(add_label).delete(remove_label))
        .route("/{id}/task-lists", post(create_task_list))
        .route("/{id}/task-lists/{tlid}", put(update_task_list).delete(delete_task_list))
        .route("/{id}/task-lists/{tlid}/tasks", post(create_task))
        .route("/{id}/task-lists/{tlid}/tasks/{tid}", put(update_task).delete(delete_task))
        .route("/{id}/comments", get(axum::routing::get(list_comments)))
        .route("/{id}/actions", get(axum::routing::get(list_actions)))
}

async fn user_id(session: &tower_sessions::Session) -> Result<String, AppError> {
    session.get("user_id").await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or(AppError::Unauthorized("not logged in".into()))
}

async fn create_card(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    Json(req): Json<CreateCardRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let uid = user_id(&session).await?;
    let id = uuid::Uuid::new_v4().to_string();

    let list = repository::list_repo::get_by_id(&state.db, &req.list_id)
        .await?
        .ok_or(AppError::NotFound("list not found".into()))?;

    let card = repository::card_repo::create(&state.db, &id, &list.board_id, &req.list_id, &req.name, &req.description, &uid).await?;

    repository::action_repo::record(&state.db, &id, Some(&uid), "createCard", serde_json::json!({"card": {"name": &req.name}})).await?;

    Ok(Json(serde_json::json!(card)))
}

async fn get_card(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    Path(card_id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let _uid = user_id(&session).await?;

    let card = repository::card_repo::get_by_id(&state.db, &card_id)
        .await?
        .ok_or(AppError::NotFound("card not found".into()))?;

    let members = repository::card_repo::list_members(&state.db, &card_id).await?;
    let labels = repository::card_repo::list_labels(&state.db, &card_id).await?;
    let comments = repository::comment_repo::list_by_card(&state.db, &card_id).await?;
    let checklists = repository::checklist_repo::list_by_card(&state.db, &card_id).await?;
    let actions = repository::action_repo::list_by_card(&state.db, &card_id).await?;

    Ok(Json(serde_json::json!({
        "card": card,
        "members": members,
        "labels": labels,
        "comments": comments,
        "checklists": checklists,
        "actions": actions,
    })))
}

async fn update_card(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    Path(card_id): Path<String>,
    Json(req): Json<UpdateCardRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let _uid = user_id(&session).await?;

    let card = repository::card_repo::update_card(&state.db, &card_id, &req).await?;

    Ok(Json(serde_json::json!(card)))
}

async fn delete_card(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    Path(card_id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let _uid = user_id(&session).await?;
    repository::card_repo::delete(&state.db, &card_id).await?;
    Ok(Json(serde_json::json!({"ok": true})))
}

async fn move_card(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    Path(card_id): Path<String>,
    Json(req): Json<MoveCardRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let uid = user_id(&session).await?;

    let old = repository::card_repo::get_by_id(&state.db, &card_id)
        .await?
        .ok_or(AppError::NotFound("card not found".into()))?;

    let card = repository::card_repo::move_card(&state.db, &card_id, &req.list_id, req.position).await?;

    repository::action_repo::record(
        &state.db, &card_id, Some(&uid), "moveCard",
        serde_json::json!({"fromList": {"id": old.list_id}, "toList": {"id": req.list_id}}),
    ).await?;

    Ok(Json(serde_json::json!(card)))
}

async fn add_member(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    Path(card_id): Path<String>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    let uid = user_id(&session).await?;
    let member_id = req["user_id"].as_str().ok_or(AppError::BadRequest("user_id required".into()))?;

    repository::card_repo::add_member(&state.db, &card_id, member_id).await?;
    repository::action_repo::record(&state.db, &card_id, Some(&uid), "addMemberToCard", serde_json::json!({"userId": member_id})).await?;

    Ok(Json(serde_json::json!({"ok": true})))
}

async fn remove_member(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    Path(card_id): Path<String>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    let _uid = user_id(&session).await?;
    let member_id = req["user_id"].as_str().ok_or(AppError::BadRequest("user_id required".into()))?;
    repository::card_repo::remove_member(&state.db, &card_id, member_id).await?;
    Ok(Json(serde_json::json!({"ok": true})))
}

async fn add_label(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    Path(card_id): Path<String>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    let _uid = user_id(&session).await?;
    let label_id = req["label_id"].as_str().ok_or(AppError::BadRequest("label_id required".into()))?;
    repository::card_repo::add_label(&state.db, &card_id, label_id).await?;
    Ok(Json(serde_json::json!({"ok": true})))
}

async fn remove_label(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    Path(card_id): Path<String>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    let _uid = user_id(&session).await?;
    let label_id = req["label_id"].as_str().ok_or(AppError::BadRequest("label_id required".into()))?;
    repository::card_repo::remove_label(&state.db, &card_id, label_id).await?;
    Ok(Json(serde_json::json!({"ok": true})))
}

// ---- Checklists ----

async fn create_task_list(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    Path(card_id): Path<String>,
    Json(req): Json<CreateTaskListRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let _uid = user_id(&session).await?;
    let id = uuid::Uuid::new_v4().to_string();
    let tl = repository::checklist_repo::create_task_list(&state.db, &id, &req.card_id, &req.name, 65536.0).await?;
    Ok(Json(serde_json::json!(tl)))
}

async fn update_task_list(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    Path((_card_id, tlid)): Path<(String, String)>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    let _uid = user_id(&session).await?;
    if let Some(name) = req["name"].as_str() {
        repository::checklist_repo::update_task_list_name(&state.db, &tlid, name).await?;
    }
    let tl = repository::checklist_repo::task_list_by_id(&state.db, &tlid)
        .await?
        .ok_or(AppError::NotFound("task list not found".into()))?;
    Ok(Json(serde_json::json!(tl)))
}

async fn delete_task_list(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    Path((_card_id, tlid)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>, AppError> {
    let _uid = user_id(&session).await?;
    repository::checklist_repo::delete_task_list(&state.db, &tlid).await?;
    Ok(Json(serde_json::json!({"ok": true})))
}

async fn create_task(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    Path((_card_id, tlid)): Path<(String, String)>,
    Json(req): Json<CreateTaskRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let _uid = user_id(&session).await?;
    let id = uuid::Uuid::new_v4().to_string();
    let task = repository::checklist_repo::create_task(&state.db, &id, &tlid, &req.name, 65536.0).await?;
    Ok(Json(serde_json::json!(task)))
}

async fn update_task(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    Path((_card_id, _tlid, tid)): Path<(String, String, String)>,
    Json(req): Json<UpdateTaskRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let _uid = user_id(&session).await?;
    let task = repository::checklist_repo::update_task(
        &state.db, &tid,
        req.name.as_deref(),
        req.is_completed,
        req.position,
        req.assignee_id.as_deref(),
    ).await?;
    Ok(Json(serde_json::json!(task)))
}

async fn delete_task(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    Path((_card_id, _tlid, tid)): Path<(String, String, String)>,
) -> Result<Json<serde_json::Value>, AppError> {
    let _uid = user_id(&session).await?;
    repository::checklist_repo::delete_task(&state.db, &tid).await?;
    Ok(Json(serde_json::json!({"ok": true})))
}

async fn list_comments(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    Path(card_id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let _uid = user_id(&session).await?;
    let comments = repository::comment_repo::list_by_card(&state.db, &card_id).await?;
    Ok(Json(serde_json::json!(comments)))
}

async fn list_actions(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    Path(card_id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let _uid = user_id(&session).await?;
    let actions = repository::action_repo::list_by_card(&state.db, &card_id).await?;
    Ok(Json(serde_json::json!(actions)))
}
