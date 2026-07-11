use axum::extract::State;
use axum::http::HeaderMap;
use axum::routing::post;
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::atomic::AtomicUsize;
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

#[derive(Clone, Serialize, Deserialize)]
pub struct ApiMessage {
    pub role: String,
    pub content: String,
    pub tool_calls: Option<Vec<ToolCall>>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub function: ToolCallFunction,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ToolCallFunction {
    pub name: String,
    pub arguments: String,
}

#[derive(Clone, Serialize)]
pub struct ToolDefinition {
    #[serde(rename = "type")]
    pub type_: String,
    pub function: serde_json::Value,
}

#[derive(Serialize)]
struct ApiRequest {
    model: String,
    messages: Vec<ApiMessage>,
    tools: Option<Vec<ToolDefinition>>,
}

#[derive(Clone, Deserialize)]
pub struct ApiChoice {
    pub message: ApiMessage,
}

#[derive(Clone, Deserialize)]
pub struct ApiResponse {
    pub choices: Vec<ApiChoice>,
}

// -- LLM client abstraction --

#[async_trait::async_trait]
pub trait LlmClient: Send + Sync {
    async fn chat(&self, api_key: &str, messages: Vec<ApiMessage>, tools: Option<Vec<ToolDefinition>>) -> Result<ApiResponse, AppError>;
}

pub struct RealLlmClient;

#[async_trait::async_trait]
impl LlmClient for RealLlmClient {
    async fn chat(&self, api_key: &str, messages: Vec<ApiMessage>, tools: Option<Vec<ToolDefinition>>) -> Result<ApiResponse, AppError> {
        let client = reqwest::Client::new();
        let body = ApiRequest { model: "deepseek-chat".into(), messages, tools };

        let resp = client.post("https://api.deepseek.com/chat/completions")
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send().await
            .map_err(|e| AppError::Internal(format!("DeepSeek request failed: {e}")))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(AppError::Internal(format!("DeepSeek returned {status}: {text}")));
        }

        resp.json().await.map_err(|e| AppError::Internal(format!("parse failed: {e}")))
    }
}

/// Returns canned responses in sequence. Panics if more calls than responses.
pub struct MockLlmClient {
    responses: Vec<Result<ApiResponse, AppError>>,
    index: AtomicUsize,
}

impl MockLlmClient {
    pub fn new(responses: Vec<Result<ApiResponse, AppError>>) -> Self {
        Self { responses, index: AtomicUsize::new(0) }
    }
}

#[async_trait::async_trait]
impl LlmClient for MockLlmClient {
    async fn chat(&self, _api_key: &str, _messages: Vec<ApiMessage>, _tools: Option<Vec<ToolDefinition>>) -> Result<ApiResponse, AppError> {
        let i = self.index.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        self.responses[i].clone()
    }
}

// -- Handler --

async fn chat(State(state): State<Arc<AppState>>, session: Session, headers: HeaderMap, Json(req): Json<ChatRequest>) -> Result<Json<ChatResponse>, AppError> {
    let user_id = crate::auth::resolve_user(&session, &headers, &state.db).await?;

    let api_key = std::env::var("DEEPSEEK_API_KEY").unwrap_or_default();
    if api_key.is_empty() {
        return Ok(Json(ChatResponse { reply: "AI assistant is not configured. Set DEEPSEEK_API_KEY in .env".into() }));
    }

    let mut api_messages: Vec<ApiMessage> = Vec::new();
    api_messages.push(ApiMessage { role: "system".into(), content: "You are a helpful assistant for a project management board. You can use tools to help users manage cards and boards.".into(), tool_calls: None });
    for msg in req.messages {
        api_messages.push(ApiMessage { role: msg.role, content: msg.content, tool_calls: None });
    }

    let tools = Some(DefaultToolDefinition::list());

    let response = state.ai_client.chat(&api_key, api_messages.clone(), tools.clone()).await?;

    if let Some(tool_calls) = response.choices.clone().into_iter().next().and_then(|c| c.message.tool_calls) {
        for call in tool_calls {
            api_messages.push(ApiMessage { role: "assistant".into(), content: "".into(), tool_calls: None });

            let args: serde_json::Value = serde_json::from_str(&call.function.arguments).unwrap_or_default();
            let result = crate::ai_tools::execute_tool(&call.function.name, &args, &user_id, &state.db).await;

            api_messages.push(ApiMessage {
                role: "tool".into(),
                content: serde_json::to_string(&result.data).unwrap_or_else(|_| "{}".into()),
                tool_calls: None,
            });
        }

        let response2 = state.ai_client.chat(&api_key, api_messages, tools).await?;
        let reply = response2.choices.into_iter().next().map(|c| c.message.content).unwrap_or_default();
        Ok(Json(ChatResponse { reply }))
    } else {
        let reply = response.choices.into_iter().next().map(|c| c.message.content).unwrap_or_default();
        Ok(Json(ChatResponse { reply }))
    }
}

// -- Tool definitions builder --

struct DefaultToolDefinition;

impl DefaultToolDefinition {
    fn list() -> Vec<ToolDefinition> {
        vec![
            ToolDefinition { type_: "function".into(), function: json!({
                "name": "create_card",
                "description": "Create a new card in a list",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "list_id": {"type": "string", "description": "ID of the list"},
                        "name": {"type": "string", "description": "Card name"},
                        "description": {"type": "string", "description": "Card description"}
                    },
                    "required": ["list_id", "name"]
                }
            })},
            ToolDefinition { type_: "function".into(), function: json!({
                "name": "search_cards",
                "description": "Search cards and boards by query",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "query": {"type": "string", "description": "Search query"}
                    },
                    "required": ["query"]
                }
            })},
            ToolDefinition { type_: "function".into(), function: json!({
                "name": "get_board_lists",
                "description": "Get all lists in a board",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "board_id": {"type": "string", "description": "Board ID"}
                    },
                    "required": ["board_id"]
                }
            })},
        ]
    }
}
