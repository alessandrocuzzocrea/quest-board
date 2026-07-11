use axum::extract::State;
use axum::http::HeaderMap;
use axum::routing::post;
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tower_sessions::Session;

use crate::error::AppError;
use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct ChatRequest {
    pub messages: Vec<Message>,
    pub board_id: Option<String>,
    pub card_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ChatResponse {
    pub reply: String,
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/chat", post(chat))
}

async fn resolve(session: Session, headers: HeaderMap, pool: &sqlx::PgPool) -> Result<uuid::Uuid, AppError> {
    crate::auth::resolve_user(&session, &headers, pool).await
}

#[derive(Serialize, Deserialize)]
struct ApiMessage {
    role: String,
    content: String,
}

#[derive(Serialize)]
struct ApiRequest {
    model: String,
    messages: Vec<ApiMessage>,
}

#[derive(Deserialize)]
struct ApiChoice {
    message: ApiMessage,
}

#[derive(Deserialize)]
struct ApiResponse {
    choices: Vec<ApiChoice>,
}

async fn chat(
    State(state): State<Arc<AppState>>,
    session: Session,
    headers: HeaderMap,
    Json(req): Json<ChatRequest>,
) -> Result<Json<ChatResponse>, AppError> {
    let _user_id = resolve(session, headers, &state.db).await?;

    let system_prompt = "You are a helpful assistant for a project management board. Help users manage their tasks, cards, and boards.";

    let api_key = std::env::var("DEEPSEEK_API_KEY").unwrap_or_default();

    if api_key.is_empty() {
        return Ok(Json(ChatResponse {
            reply: "AI assistant is not configured. Set DEEPSEEK_API_KEY in .env".into(),
        }));
    }

    let mut api_messages = Vec::new();
    api_messages.push(ApiMessage { role: "system".into(), content: system_prompt.into() });
    for msg in req.messages {
        api_messages.push(ApiMessage { role: msg.role, content: msg.content });
    }

    let body = ApiRequest { model: "deepseek-chat".into(), messages: api_messages };

    let client = reqwest::Client::new();
    let resp = client
        .post("https://api.deepseek.com/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| AppError::Internal(format!("DeepSeek request failed: {e}")))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_else(|_| "unknown error".into());
        return Err(AppError::Internal(format!("DeepSeek returned {status}: {text}")));
    }

    let data: ApiResponse = resp.json().await.map_err(|e| AppError::Internal(format!("failed to parse DeepSeek response: {e}")))?;
    let reply = data.choices.into_iter().next().map(|c| c.message.content).unwrap_or_default();

    Ok(Json(ChatResponse { reply }))
}
