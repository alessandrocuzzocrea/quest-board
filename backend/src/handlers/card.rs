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

async fn user_id(session: &tower_sessions::Session) -> Result<uuid::Uuid, AppError> {
    let uid: String = session.get("user_id").await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or(AppError::Unauthorized("not logged in".into()))?;
    uuid::Uuid::parse_str(&uid).map_err(|_| AppError::Internal("invalid user id".into()))
}

#[utoipa::path(
    post,
    path = "/cards/",
    tag = "cards",
    request_body = CreateCardRequest,
    responses(
        (status = 200, description = "Card created", body = serde_json::Value),
        (status = 401, description = "Unauthorized")
    )
)]
 
async fn create_card(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    Json(req): Json<CreateCardRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let uid = user_id(&session).await?;

    let list = repository::list_repo::get_by_id(&state.db, &req.list_id)
        .await?
        .ok_or(AppError::NotFound("list not found".into()))?;

    let card = repository::card_repo::create(&state.db, &list.board_id, &req.list_id, &req.name, &req.description, &uid).await?;

    repository::action_repo::record(&state.db, &card.id, Some(&uid), "createCard", serde_json::json!({"card": {"name": &req.name}})).await?;

    Ok(Json(serde_json::json!(card)))

}
#[utoipa::path(
    get,
    path = "/cards/{id}",
    tag = "cards",
    params(
        ("id" = String, Path, description = "Card ID")
    ),
    responses(
        (status = 200, description = "Card details", body = serde_json::Value),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Card not found")
    )
)]
 
async fn get_card(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    Path(card_id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let _uid = user_id(&session).await?;
    let card_id: uuid::Uuid = card_id.parse().map_err(|_| AppError::BadRequest("invalid card id".into()))?;

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

#[utoipa::path(
    put,
    path = "/cards/{id}",
    tag = "cards",
    request_body = UpdateCardRequest,
    params(
        ("id" = String, Path, description = "Card ID")
    ),
    responses(
        (status = 200, description = "Card updated", body = serde_json::Value),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Card not found")
    )
)]
 
async fn update_card(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    Path(card_id): Path<String>,
    Json(req): Json<UpdateCardRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let _uid = user_id(&session).await?;
    let card_id: uuid::Uuid = card_id.parse().map_err(|_| AppError::BadRequest("invalid card id".into()))?;

    let card = repository::card_repo::update_card(&state.db, &card_id, &req).await?;

    Ok(Json(serde_json::json!(card)))
}

#[utoipa::path(
    delete,
    path = "/cards/{id}",
    tag = "cards",
    params(
        ("id" = String, Path, description = "Card ID")
    ),
    responses(
        (status = 200, description = "Card deleted"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Card not found")
    )
)]
 
async fn delete_card(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    Path(card_id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let _uid = user_id(&session).await?;
    let card_id: uuid::Uuid = card_id.parse().map_err(|_| AppError::BadRequest("invalid card id".into()))?;
    repository::card_repo::delete(&state.db, &card_id).await?;
    Ok(Json(serde_json::json!({"ok": true})))
}

#[utoipa::path(
    put,
    path = "/cards/{id}/move",
    tag = "cards",
    request_body = MoveCardRequest,
    params(
        ("id" = String, Path, description = "Card ID")
    ),
    responses(
        (status = 200, description = "Card moved", body = serde_json::Value),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Card not found")
    )
)]
 
async fn move_card(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    Path(card_id): Path<String>,
    Json(req): Json<MoveCardRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let uid = user_id(&session).await?;
    let card_id: uuid::Uuid = card_id.parse().map_err(|_| AppError::BadRequest("invalid card id".into()))?;

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

#[utoipa::path(
    post,
    path = "/cards/{id}/members",
    tag = "cards",
    request_body(content = serde_json::Value, description = "Member data"),
    params(
        ("id" = String, Path, description = "Card ID")
    ),
    responses(
        (status = 200, description = "Member added"),
        (status = 401, description = "Unauthorized")
    )
)]
 
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

    repository::card_repo::add_member(&state.db, &card_id, &member_id).await?;
    repository::action_repo::record(&state.db, &card_id, Some(&uid), "addMemberToCard", serde_json::json!({"userId": member_id_str})).await?;

    Ok(Json(serde_json::json!({"ok": true})))
}

#[utoipa::path(
    delete,
    path = "/cards/{id}/members",
    tag = "cards",
    request_body(content = serde_json::Value, description = "Member data"),
    params(
        ("id" = String, Path, description = "Card ID")
    ),
    responses(
        (status = 200, description = "Member removed"),
        (status = 401, description = "Unauthorized")
    )
)]
 
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
    repository::card_repo::remove_member(&state.db, &card_id, &member_id).await?;
    Ok(Json(serde_json::json!({"ok": true})))
}

#[utoipa::path(
    post,
    path = "/cards/{id}/labels",
    tag = "cards",
    request_body(content = serde_json::Value, description = "Label data"),
    params(
        ("id" = String, Path, description = "Card ID")
    ),
    responses(
        (status = 200, description = "Label added"),
        (status = 401, description = "Unauthorized")
    )
)]
 
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
    repository::card_repo::add_label(&state.db, &card_id, &label_id).await?;
    Ok(Json(serde_json::json!({"ok": true})))
}

#[utoipa::path(
    delete,
    path = "/cards/{id}/labels",
    tag = "cards",
    request_body(content = serde_json::Value, description = "Label data"),
    params(
        ("id" = String, Path, description = "Card ID")
    ),
    responses(
        (status = 200, description = "Label removed"),
        (status = 401, description = "Unauthorized")
    )
)]
 
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
    repository::card_repo::remove_label(&state.db, &card_id, &label_id).await?;
    Ok(Json(serde_json::json!({"ok": true})))
}

// ---- Checklists ----

#[utoipa::path(
    post,
    path = "/cards/{id}/task-lists",
    tag = "cards",
    request_body = CreateTaskListRequest,
    params(
        ("id" = String, Path, description = "Card ID")
    ),
    responses(
        (status = 200, description = "Task list created", body = serde_json::Value),
        (status = 401, description = "Unauthorized")
    )
)]
 
async fn create_task_list(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    Path(_card_id): Path<String>,
    Json(req): Json<CreateTaskListRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let _uid = user_id(&session).await?;
    let tl = repository::checklist_repo::create_task_list(&state.db, &req.card_id, &req.name, 65536.0).await?;
    Ok(Json(serde_json::json!(tl)))
}

#[utoipa::path(
    put,
    path = "/cards/{id}/task-lists/{tlid}",
    tag = "cards",
    request_body(content = serde_json::Value, description = "Task list update data"),
    params(
        ("id" = String, Path, description = "Card ID"),
        ("tlid" = String, Path, description = "Task list ID")
    ),
    responses(
        (status = 200, description = "Task list updated", body = serde_json::Value),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Task list not found")
    )
)]
 
async fn update_task_list(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    Path((_card_id, tlid)): Path<(String, String)>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    let _uid = user_id(&session).await?;
    let tlid: uuid::Uuid = tlid.parse().map_err(|_| AppError::BadRequest("invalid task list id".into()))?;
    if let Some(name) = req["name"].as_str() {
        repository::checklist_repo::update_task_list_name(&state.db, &tlid, name).await?;
    }
    let tl = repository::checklist_repo::task_list_by_id(&state.db, &tlid)
        .await?
        .ok_or(AppError::NotFound("task list not found".into()))?;
    Ok(Json(serde_json::json!(tl)))
}

#[utoipa::path(
    delete,
    path = "/cards/{id}/task-lists/{tlid}",
    tag = "cards",
    params(
        ("id" = String, Path, description = "Card ID"),
        ("tlid" = String, Path, description = "Task list ID")
    ),
    responses(
        (status = 200, description = "Task list deleted"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Task list not found")
    )
)]
 
async fn delete_task_list(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    Path((_card_id, tlid)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>, AppError> {
    let _uid = user_id(&session).await?;
    let tlid: uuid::Uuid = tlid.parse().map_err(|_| AppError::BadRequest("invalid task list id".into()))?;
    repository::checklist_repo::delete_task_list(&state.db, &tlid).await?;
    Ok(Json(serde_json::json!({"ok": true})))
}

#[utoipa::path(
    post,
    path = "/cards/{id}/task-lists/{tlid}/tasks",
    tag = "cards",
    request_body = CreateTaskRequest,
    params(
        ("id" = String, Path, description = "Card ID"),
        ("tlid" = String, Path, description = "Task list ID")
    ),
    responses(
        (status = 200, description = "Task created", body = serde_json::Value),
        (status = 401, description = "Unauthorized")
    )
)]
 
async fn create_task(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    Path((_card_id, tlid)): Path<(String, String)>,
    Json(req): Json<CreateTaskRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let _uid = user_id(&session).await?;
    let tlid: uuid::Uuid = tlid.parse().map_err(|_| AppError::BadRequest("invalid task list id".into()))?;
    let task = repository::checklist_repo::create_task(&state.db, &tlid, &req.name, 65536.0).await?;
    Ok(Json(serde_json::json!(task)))
}

#[utoipa::path(
    put,
    path = "/cards/{id}/task-lists/{tlid}/tasks/{tid}",
    tag = "cards",
    request_body = UpdateTaskRequest,
    params(
        ("id" = String, Path, description = "Card ID"),
        ("tlid" = String, Path, description = "Task list ID"),
        ("tid" = String, Path, description = "Task ID")
    ),
    responses(
        (status = 200, description = "Task updated", body = serde_json::Value),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Task not found")
    )
)]
 
async fn update_task(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    Path((_card_id, _tlid, tid)): Path<(String, String, String)>,
    Json(req): Json<UpdateTaskRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let _uid = user_id(&session).await?;
    let tid: uuid::Uuid = tid.parse().map_err(|_| AppError::BadRequest("invalid task id".into()))?;
    let task = repository::checklist_repo::update_task(
        &state.db, &tid,
        req.name.as_deref(),
        req.is_completed,
        req.position,
        req.assignee_id.as_ref(),
    ).await?;
    Ok(Json(serde_json::json!(task)))
}

#[utoipa::path(
    delete,
    path = "/cards/{id}/task-lists/{tlid}/tasks/{tid}",
    tag = "cards",
    params(
        ("id" = String, Path, description = "Card ID"),
        ("tlid" = String, Path, description = "Task list ID"),
        ("tid" = String, Path, description = "Task ID")
    ),
    responses(
        (status = 200, description = "Task deleted"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Task not found")
    )
)]
 
async fn delete_task(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    Path((_card_id, _tlid, tid)): Path<(String, String, String)>,
) -> Result<Json<serde_json::Value>, AppError> {
    let _uid = user_id(&session).await?;
    let tid: uuid::Uuid = tid.parse().map_err(|_| AppError::BadRequest("invalid task id".into()))?;
    repository::checklist_repo::delete_task(&state.db, &tid).await?;
    Ok(Json(serde_json::json!({"ok": true})))
}

#[utoipa::path(
    get,
    path = "/cards/{id}/comments",
    tag = "cards",
    operation_id = "cards_list_comments",
    params(
        ("id" = String, Path, description = "Card ID")
    ),
    responses(
        (status = 200, description = "List of comments", body = serde_json::Value),
        (status = 401, description = "Unauthorized")
    )
)]
 
async fn list_comments(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    Path(card_id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let _uid = user_id(&session).await?;
    let card_id: uuid::Uuid = card_id.parse().map_err(|_| AppError::BadRequest("invalid card id".into()))?;
    let comments = repository::comment_repo::list_by_card(&state.db, &card_id).await?;
    Ok(Json(serde_json::json!(comments)))
}

#[utoipa::path(
    get,
    path = "/cards/{id}/actions",
    tag = "cards",
    params(
        ("id" = String, Path, description = "Card ID")
    ),
    responses(
        (status = 200, description = "List of actions", body = serde_json::Value),
        (status = 401, description = "Unauthorized")
    )
)]
async fn list_actions(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    Path(card_id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let _uid = user_id(&session).await?;
    let card_id: uuid::Uuid = card_id.parse().map_err(|_| AppError::BadRequest("invalid card id".into()))?;
    let actions = repository::action_repo::list_by_card(&state.db, &card_id).await?;
    Ok(Json(serde_json::json!(actions)))
}
