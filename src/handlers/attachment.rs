use axum::extract::{Multipart, Path, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use std::sync::Arc;
use uuid::Uuid;

use crate::error::AppError;
use crate::services::AttachmentService;
use crate::AppState;

const MAX_UPLOAD_SIZE: usize = 10 * 1024 * 1024; // 10 MB

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/card/{card_id}", get(list_attachments))
        .route("/link", post(create_link_attachment))
        .route("/upload", post(upload_attachment))
        .route("/{id}", axum::routing::delete(delete_attachment))
}

async fn user_id(session: &tower_sessions::Session) -> Result<Uuid, AppError> {
    let uid: String = session.get("user_id").await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or(AppError::Unauthorized("not logged in".into()))?;
    Uuid::parse_str(&uid).map_err(|_| AppError::Internal("invalid user id".into()))
}

#[utoipa::path(
    get,
    path = "/api/v1/attachments/card/{card_id}",
    tag = "attachments",
    params(("card_id" = String, Path)),
    responses(
        (status = 200, body = serde_json::Value)
    )
)]
async fn list_attachments(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    Path(card_id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let _uid = user_id(&session).await?;
    let card_id: Uuid = card_id.parse().map_err(|_| AppError::BadRequest("invalid card_id".into()))?;
    let svc = AttachmentService::new(state.db.clone());
    let attachments = svc.list_by_card(&card_id).await?;
    Ok(Json(serde_json::json!(attachments)))
}

#[utoipa::path(
    post,
    path = "/api/v1/attachments/link",
    tag = "attachments",
    request_body = serde_json::Value,
    responses(
        (status = 200, body = serde_json::Value)
    )
)]
async fn create_link_attachment(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    let uid = user_id(&session).await?;
    let card_id_str = req["card_id"].as_str().ok_or(AppError::BadRequest("card_id required".into()))?;
    let card_id: Uuid = card_id_str.parse().map_err(|_| AppError::BadRequest("invalid card_id".into()))?;
    let name = req["name"].as_str().unwrap_or("link");
    let url = req["url"].as_str().ok_or(AppError::BadRequest("url required".into()))?;

    let svc = AttachmentService::new(state.db.clone());
    let attachment = svc.create_link(&card_id, &uid, name, url).await?;
    Ok(Json(serde_json::json!(attachment)))
}

async fn upload_attachment(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    mut multipart: Multipart,
) -> Result<Json<serde_json::Value>, AppError> {
    let uid = user_id(&session).await?;
    let mut card_id: Option<Uuid> = None;
    let mut file_data: Option<(String, Vec<u8>, String)> = None;

    while let Some(field) = multipart.next_field().await.map_err(|e| AppError::BadRequest(e.to_string()))? {
        let name = field.name().unwrap_or("").to_string();
        match name.as_str() {
            "card_id" => {
                let val = field.text().await.map_err(|e| AppError::BadRequest(e.to_string()))?;
                card_id = Some(val.parse().map_err(|_| AppError::BadRequest("invalid card_id".into()))?);
            }
            "file" => {
                let filename = field.file_name().unwrap_or("file").to_string();
                let mime = field.content_type().map(|m| m.to_string()).unwrap_or_else(|| "application/octet-stream".into());
                let data = field.bytes().await.map_err(|e| AppError::BadRequest(e.to_string()))?;
                if data.len() > MAX_UPLOAD_SIZE {
                    return Err(AppError::BadRequest("file too large (max 10 MB)".into()));
                }
                file_data = Some((filename, data.to_vec(), mime));
            }
            _ => continue,
        }
    }

    let card_id = card_id.ok_or(AppError::BadRequest("card_id required".into()))?;
    let (filename, bytes, mime) = file_data.ok_or(AppError::BadRequest("file required".into()))?;
    let unique_name = format!("{}_{}", Uuid::new_v4(), filename);
    let file_path = format!("uploads/{}", unique_name);

    tokio::fs::create_dir_all("uploads").await
        .map_err(|e| AppError::Internal(format!("failed to create uploads dir: {e}")))?;
    tokio::fs::write(&file_path, &bytes).await
        .map_err(|e| AppError::Internal(format!("failed to write file: {e}")))?;

    let svc = AttachmentService::new(state.db.clone());
    let attachment = svc.create_file(&card_id, &uid, &filename, &file_path, bytes.len() as i64, &mime).await?;
    Ok(Json(serde_json::json!(attachment)))
}

#[utoipa::path(
    delete,
    path = "/api/v1/attachments/{id}",
    tag = "attachments",
    params(("id" = String, Path)),
    responses(
        (status = 200, body = serde_json::Value)
    )
)]

async fn delete_attachment(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    Path(attachment_id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let _uid = user_id(&session).await?;
    let attachment_id: Uuid = attachment_id.parse().map_err(|_| AppError::BadRequest("invalid attachment_id".into()))?;
    let svc = AttachmentService::new(state.db.clone());
    svc.delete(&attachment_id).await?;
    Ok(Json(serde_json::json!({"ok": true})))
}
