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

async fn resolve(
    session: Session,
    headers: HeaderMap,
    pool: &sqlx::PgPool,
) -> Result<uuid::Uuid, AppError> {
    crate::auth::resolve_user(&session, &headers, pool).await
}

async fn chat(
    State(state): State<Arc<AppState>>,
    session: Session,
    headers: HeaderMap,
    Json(req): Json<ChatRequest>,
) -> Result<Json<ChatResponse>, AppError> {
    // Authenticate
    let _user_id = resolve(session, headers, &state.db).await?;

    // Build context string for the system prompt
    let mut context_parts: Vec<String> = Vec::new();

    if let Some(board_id) = &req.board_id {
        if let Ok(uid) = uuid::Uuid::parse_str(board_id) {
            let row: Option<(String, String)> = sqlx::query_as(
                "SELECT title, description FROM boards WHERE id = $1",
            )
            .bind(uid)
            .fetch_optional(&state.db)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;

            if let Some((title, description)) = row {
                context_parts.push(format!(
                    "Board: {} — {}",
                    title,
                    description
                ));
            }
        }
    }

    if let Some(card_id) = &req.card_id {
        if let Ok(uid) = uuid::Uuid::parse_str(card_id) {
            let row: Option<(String, String)> = sqlx::query_as(
                "SELECT title, description FROM cards WHERE id = $1",
            )
            .bind(uid)
            .fetch_optional(&state.db)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;

            if let Some((title, description)) = row {
                context_parts.push(format!(
                    "Card: {} — {}",
                    title,
                    description
                ));
            }
        }
    }

    // Build system prompt
    let mut system_prompt = String::from(
        "You are a helpful assistant helping with a project management board.",
    );
    if !context_parts.is_empty() {
        system_prompt.push_str("\n\nCurrent context:\n");
        system_prompt.push_str(&context_parts.join("\n"));
    }

    let api_key = std::env::var("CHATGPT_API_KEY").unwrap_or_default();

    if api_key.is_empty() {
        return Ok(Json(ChatResponse {
            reply: "AI assistant is not configured. Set CHATGPT_API_KEY in .env".into(),
        }));
    }

    #[derive(Serialize, Deserialize)]
    struct OpenAiMessage {
        role: String,
        content: String,
    }

    #[derive(Serialize)]
    struct OpenAiRequest {
        model: String,
        messages: Vec<OpenAiMessage>,
    }

    #[derive(Deserialize)]
    struct OpenAiChoice {
        message: OpenAiMessage,
    }

    #[derive(Deserialize)]
    struct OpenAiResponse {
        choices: Vec<OpenAiChoice>,
    }

    let mut openai_messages = Vec::new();
    openai_messages.push(OpenAiMessage {
        role: "system".into(),
        content: system_prompt,
    });
    for msg in req.messages {
        openai_messages.push(OpenAiMessage {
            role: msg.role,
            content: msg.content,
        });
    }

    let body = OpenAiRequest {
        model: "gpt-4o-mini".into(),
        messages: openai_messages,
    };

    let client = reqwest::Client::new();
    let resp = client
        .post("https://api.openai.com/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| AppError::Internal(format!("OpenAI request failed: {e}")))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp
            .text()
            .await
            .unwrap_or_else(|_| "unknown error".into());
        return Err(AppError::Internal(format!(
            "OpenAI returned {status}: {text}"
        )));
    }

    let data: OpenAiResponse = resp
        .json()
        .await
        .map_err(|e| AppError::Internal(format!("failed to parse OpenAI response: {e}")))?;

    let reply = data
        .choices
        .into_iter()
        .next()
        .map(|c| c.message.content)
        .unwrap_or_default();

    Ok(Json(ChatResponse { reply }))
}
