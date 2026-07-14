use quest_board::handlers::ai::{ApiChoice, ApiMessage, ApiResponse, MockLlmClient, ToolCall, ToolCallFunction};
use quest_board::AppState;
use std::sync::Arc;
use std::sync::LazyLock;
use tokio::sync::Mutex;
use tower::ServiceExt;

static SETUP_MUTEX: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

async fn setup() -> (axum::Router, sqlx::PgPool) {
    let _guard = SETUP_MUTEX.lock().await;
    dotenvy::from_filename(".env.test").ok();

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:quest@localhost:5432/quest_test".into());

    let pool = sqlx::PgPool::connect(&database_url).await.unwrap();
    sqlx::query("DROP TABLE IF EXISTS api_keys, sessions, favorites, notifications, actions, tasks, task_lists, attachments, comments, card_labels, labels, card_members, cards, lists, board_members, boards, users CASCADE")
        .execute(&pool).await.ok();
    quest_board::db::run_migrations(&pool).await.unwrap();

    let mock = MockLlmClient::new(vec![
        Ok(ApiResponse {
            choices: vec![ApiChoice {
                message: ApiMessage {
                    role: "assistant".to_string(),
                    content: "".to_string(),
                    tool_calls: Some(vec![ToolCall {
                        id: "call_1".to_string(),
                        type_: "function".to_string(),
                        function: ToolCallFunction {
                            name: "search_cards".to_string(),
                            arguments: r#"{"query":"cards"}"#.to_string(),
                        },
                    }]),
                },
            }],
        }),
        Ok(ApiResponse {
            choices: vec![ApiChoice {
                message: ApiMessage {
                    role: "assistant".to_string(),
                    content: "I found some cards for you. card results as requested.".to_string(),
                    tool_calls: None,
                },
            }],
        }),
    ]);
    let state = Arc::new(AppState {
        db: pool.clone(),
        ai_client: Arc::new(mock),
    });
    let app = quest_board::build_app(pool.clone(), state).await;
    (app, pool)
}

#[tokio::test]
async fn test_ai_chat_requires_auth() {
    let (app, _pool) = setup().await;
    let body = r#"{"messages":[{"role":"user","content":"hello"}]}"#;
    let req = axum::http::Request::builder()
        .method("POST").uri("/api/v1/ai/chat")
        .header("content-type", "application/json")
        .body(axum::body::Body::from(body)).unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 401, "unauthenticated chat should be rejected");
}

#[tokio::test]
async fn test_ai_chat_returns_not_configured() {
    let (app, _pool) = setup().await;

    // Register and login
    let req = axum::http::Request::builder()
        .method("POST").uri("/api/v1/auth/register")
        .header("content-type", "application/json")
        .body(axum::body::Body::from(r#"{"email":"ai@test.com","password":"pass","name":"AI Tester"}"#)).unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    let cookie = resp.headers().get("set-cookie").and_then(|v| v.to_str().ok())
        .map(|s| s.split(';').next().unwrap_or("").to_string()).unwrap();

    // Ensure DEEPSEEK_API_KEY is NOT set
    std::env::remove_var("DEEPSEEK_API_KEY");

    let body = r#"{"messages":[{"role":"user","content":"hello"}]}"#;
    let req = axum::http::Request::builder()
        .method("POST").uri("/api/v1/ai/chat")
        .header("content-type", "application/json")
        .header("cookie", &cookie)
        .body(axum::body::Body::from(body)).unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200, "chat should return 200 even without API key");

    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert!(json["reply"].as_str().unwrap_or("").contains("not configured"),
        "should say not configured, got: {:?}", json["reply"]);
}

#[tokio::test]
async fn test_ai_chat_with_mock_tool_call() {
    let (app, _pool) = setup().await;

    // Register and get cookie
    let req = axum::http::Request::builder()
        .method("POST").uri("/api/v1/auth/register")
        .header("content-type", "application/json")
        .body(axum::body::Body::from(r#"{"email":"tool@test.com","password":"pass","name":"Tool Tester"}"#)).unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    let cookie = resp.headers().get("set-cookie").and_then(|v| v.to_str().ok())
        .map(|s| s.split(';').next().unwrap_or("").to_string()).unwrap();

    std::env::set_var("DEEPSEEK_API_KEY", "mock-key");

    let body = r#"{"messages":[{"role":"user","content":"create a card"}]}"#;
    let req = axum::http::Request::builder()
        .method("POST").uri("/api/v1/ai/chat")
        .header("content-type", "application/json")
        .header("cookie", &cookie)
        .body(axum::body::Body::from(body)).unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), 200);

    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    let reply = json["reply"].as_str().unwrap_or("");
    // The mock returns "card created" on the second call
    assert!(reply.contains("card"), "expected card reference in reply, got: {}", reply);
}
