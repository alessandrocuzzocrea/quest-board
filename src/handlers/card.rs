use axum::extract::{Path, State};
use axum::routing::{get, post, put};
use axum::{Json, Router};
use std::sync::Arc;

use crate::error::AppError;
use crate::models::card::{CreateCardRequest, MoveCardRequest, UpdateCardRequest};
use crate::models::checklist::{CreateTaskListRequest, CreateTaskRequest, UpdateTaskRequest};
use crate::services::CardService;
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

async fn user_id(session: &tower_sessions::Session) -> Result<uuid::Uuid, AppError> {
    let uid: String = session.get("user_id").await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or(AppError::Unauthorized("not logged in".into()))?;
    uuid::Uuid::parse_str(&uid).map_err(|_| AppError::Internal("invalid user id".into()))
}

async fn create_card(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    Json(req): Json<CreateCardRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let uid = user_id(&session).await?;
    let svc = CardService::new(state.db.clone(), state.event_tx.clone());
    let card = svc.create(&req, &uid).await?;
    Ok(Json(serde_json::json!(card)))
}

async fn get_card(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    Path(card_id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let _uid = user_id(&session).await?;
    let card_id: uuid::Uuid = card_id.parse().map_err(|_| AppError::BadRequest("invalid card id".into()))?;
    let svc = CardService::new(state.db.clone(), state.event_tx.clone());
    let result = svc.get_with_relations(&card_id).await?;
    Ok(Json(result))
}

async fn update_card(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    Path(card_id): Path<String>,
    Json(req): Json<UpdateCardRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let uid = user_id(&session).await?;
    let card_id: uuid::Uuid = card_id.parse().map_err(|_| AppError::BadRequest("invalid card id".into()))?;
    let svc = CardService::new(state.db.clone(), state.event_tx.clone());
    let card = svc.update(&card_id, &req, &uid).await?;
    Ok(Json(serde_json::json!(card)))
}

async fn delete_card(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    Path(card_id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let uid = user_id(&session).await?;
    let card_id: uuid::Uuid = card_id.parse().map_err(|_| AppError::BadRequest("invalid card id".into()))?;
    let svc = CardService::new(state.db.clone(), state.event_tx.clone());
    svc.delete(&card_id, &uid).await?;
    Ok(Json(serde_json::json!({"ok": true})))
}

async fn move_card(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    Path(card_id): Path<String>,
    Json(req): Json<MoveCardRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let uid = user_id(&session).await?;
    let card_id: uuid::Uuid = card_id.parse().map_err(|_| AppError::BadRequest("invalid card id".into()))?;
    let svc = CardService::new(state.db.clone(), state.event_tx.clone());
    let card = svc.move_card(&card_id, &req, &uid).await?;
    Ok(Json(serde_json::json!(card)))
}

async fn add_member(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    Path(card_id): Path<String>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    let uid = user_id(&session).await?;
    let card_id: uuid::Uuid = card_id.parse().map_err(|_| AppError::BadRequest("invalid card id".into()))?;
    let member_id_str = req["user_id"].as_str().ok_or(AppError::BadRequest("user_id required".into()))?;
    let member_id = uuid::Uuid::parse_str(member_id_str).map_err(|_| AppError::BadRequest("invalid user id".into()))?;
    let svc = CardService::new(state.db.clone(), state.event_tx.clone());
    svc.add_member(&card_id, &member_id, &uid).await?;
    Ok(Json(serde_json::json!({"ok": true})))
}

async fn remove_member(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    Path(card_id): Path<String>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    let _uid = user_id(&session).await?;
    let card_id: uuid::Uuid = card_id.parse().map_err(|_| AppError::BadRequest("invalid card id".into()))?;
    let member_id_str = req["user_id"].as_str().ok_or(AppError::BadRequest("user_id required".into()))?;
    let member_id = uuid::Uuid::parse_str(member_id_str).map_err(|_| AppError::BadRequest("invalid user id".into()))?;
    let svc = CardService::new(state.db.clone(), state.event_tx.clone());
    svc.remove_member(&card_id, &member_id).await?;
    Ok(Json(serde_json::json!({"ok": true})))
}

async fn add_label(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    Path(card_id): Path<String>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    let _uid = user_id(&session).await?;
    let card_id: uuid::Uuid = card_id.parse().map_err(|_| AppError::BadRequest("invalid card id".into()))?;
    let label_id_str = req["label_id"].as_str().ok_or(AppError::BadRequest("label_id required".into()))?;
    let label_id = uuid::Uuid::parse_str(label_id_str).map_err(|_| AppError::BadRequest("invalid label id".into()))?;
    let svc = CardService::new(state.db.clone(), state.event_tx.clone());
    svc.add_label(&card_id, &label_id).await?;
    Ok(Json(serde_json::json!({"ok": true})))
}

async fn remove_label(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    Path(card_id): Path<String>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    let _uid = user_id(&session).await?;
    let card_id: uuid::Uuid = card_id.parse().map_err(|_| AppError::BadRequest("invalid card id".into()))?;
    let label_id_str = req["label_id"].as_str().ok_or(AppError::BadRequest("label_id required".into()))?;
    let label_id = uuid::Uuid::parse_str(label_id_str).map_err(|_| AppError::BadRequest("invalid label id".into()))?;
    let svc = CardService::new(state.db.clone(), state.event_tx.clone());
    svc.remove_label(&card_id, &label_id).await?;
    Ok(Json(serde_json::json!({"ok": true})))
}

// ---- Checklists ----

async fn create_task_list(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    Path(_card_id): Path<String>,
    Json(req): Json<CreateTaskListRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let _uid = user_id(&session).await?;
    let svc = CardService::new(state.db.clone(), state.event_tx.clone());
    let tl = svc.create_task_list(&req).await?;
    Ok(Json(serde_json::json!(tl)))
}

async fn update_task_list(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    Path((_card_id, tlid)): Path<(String, String)>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    let _uid = user_id(&session).await?;
    let tlid: uuid::Uuid = tlid.parse().map_err(|_| AppError::BadRequest("invalid task list id".into()))?;
    let svc = CardService::new(state.db.clone(), state.event_tx.clone());
    let name = req["name"].as_str().unwrap_or("");
    let tl = svc.update_task_list(&tlid, name).await?;
    Ok(Json(serde_json::json!(tl)))
}

async fn delete_task_list(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    Path((_card_id, tlid)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>, AppError> {
    let _uid = user_id(&session).await?;
    let tlid: uuid::Uuid = tlid.parse().map_err(|_| AppError::BadRequest("invalid task list id".into()))?;
    let svc = CardService::new(state.db.clone(), state.event_tx.clone());
    svc.delete_task_list(&tlid).await?;
    Ok(Json(serde_json::json!({"ok": true})))
}

async fn create_task(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    Path((_card_id, tlid)): Path<(String, String)>,
    Json(req): Json<CreateTaskRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let _uid = user_id(&session).await?;
    let tlid: uuid::Uuid = tlid.parse().map_err(|_| AppError::BadRequest("invalid task list id".into()))?;
    let svc = CardService::new(state.db.clone(), state.event_tx.clone());
    let task = svc.create_task(&tlid, &req).await?;
    Ok(Json(serde_json::json!(task)))
}

async fn update_task(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    Path((_card_id, _tlid, tid)): Path<(String, String, String)>,
    Json(req): Json<UpdateTaskRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let _uid = user_id(&session).await?;
    let tid: uuid::Uuid = tid.parse().map_err(|_| AppError::BadRequest("invalid task id".into()))?;
    let svc = CardService::new(state.db.clone(), state.event_tx.clone());
    let task = svc.update_task(&tid, &req).await?;
    Ok(Json(serde_json::json!(task)))
}

async fn delete_task(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    Path((_card_id, _tlid, tid)): Path<(String, String, String)>,
) -> Result<Json<serde_json::Value>, AppError> {
    let _uid = user_id(&session).await?;
    let tid: uuid::Uuid = tid.parse().map_err(|_| AppError::BadRequest("invalid task id".into()))?;
    let svc = CardService::new(state.db.clone(), state.event_tx.clone());
    svc.delete_task(&tid).await?;
    Ok(Json(serde_json::json!({"ok": true})))
}

async fn list_comments(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    Path(card_id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let _uid = user_id(&session).await?;
    let card_id: uuid::Uuid = card_id.parse().map_err(|_| AppError::BadRequest("invalid card id".into()))?;
    let svc = CardService::new(state.db.clone(), state.event_tx.clone());
    let comments = svc.list_comments(&card_id).await?;
    Ok(Json(serde_json::json!(comments)))
}

async fn list_actions(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    Path(card_id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let _uid = user_id(&session).await?;
    let card_id: uuid::Uuid = card_id.parse().map_err(|_| AppError::BadRequest("invalid card id".into()))?;
    let svc = CardService::new(state.db.clone(), state.event_tx.clone());
    let actions = svc.list_actions(&card_id).await?;
    Ok(Json(serde_json::json!(actions)))
}
