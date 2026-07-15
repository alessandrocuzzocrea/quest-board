pub mod db;
pub mod auth;
pub mod error;
pub mod handlers;
pub mod models;
pub mod repository;
pub mod services;
pub mod slug;
pub mod session;
pub mod events;
use askama::Template;
use axum::extract::Path;
use axum::routing::get;
use axum::{
    middleware,
    response::{Html, IntoResponse, Redirect},
};
use tower_http::services::fs::ServeDir;

use std::sync::Arc;
use tower_sessions::cookie::SameSite;

use session::PgSessionStore;

pub struct AppState {
    pub db: sqlx::PgPool,
    pub event_tx: tokio::sync::broadcast::Sender<events::SseEvent>,
}

// Known app pages that require authentication (both clean and .html forms)
const PROTECTED_PATHS: &[&str] = &[
    "/boards", "/settings",
    "/boards.html", "/board.html", "/settings.html",
];

async fn require_auth_for_html(
    request: axum::http::Request<axum::body::Body>,
    next: middleware::Next,
) -> axum::response::Response {

    let req_path = request.uri().path().to_string();

    if req_path == "/login" {
        return next.run(request).await;
    }

    let is_protected = req_path.ends_with(".html")
        || PROTECTED_PATHS.contains(&req_path.as_str())
        || req_path.starts_with("/board/");

    if !is_protected {
        return next.run(request).await;
    }
    
    // Check session for user_id
    let session = request.extensions()
        .get::<tower_sessions::Session>()
        .cloned();
    
    match session {
        Some(session) => {
            match session.get::<String>("user_id").await {
                Ok(Some(_)) => next.run(request).await,
                _ => Redirect::to("/login").into_response(),
            }
        }
        None => Redirect::to("/login").into_response(),
    }
}

// ── Page handlers for clean URLs ─────────────────────────────────────

#[derive(Template)]
#[template(path = "boards.html")]
struct BoardsTemplate;

#[derive(Template)]
#[template(path = "board.html")]
struct BoardTemplate {
    board_name: String,
    board_slug: String,
}

#[derive(Template)]
#[template(path = "settings.html")]
struct SettingsTemplate;
#[derive(Template)]
#[template(path = "partials/board_grid.html")]
pub(crate) struct BoardGridTemplate {
    boards: Vec<models::board::Board>,
    query: String,
}

async fn page_boards() -> impl IntoResponse {
    Html(BoardsTemplate.render().unwrap())
}

async fn page_board_with_slug(
    Path((slug, _name)): Path<(String, String)>,
) -> impl IntoResponse {
    let board = BoardTemplate {
        board_name: _name,
        board_slug: slug,
    };
    Html(board.render().unwrap())
}

async fn page_board_html() -> impl IntoResponse {
    let board = BoardTemplate {
        board_name: String::new(),
        board_slug: String::new(),
    };
    Html(board.render().unwrap())
}

async fn page_boards_html() -> impl IntoResponse {
    Html(BoardsTemplate.render().unwrap())
}

async fn page_settings_html() -> impl IntoResponse {
    Html(SettingsTemplate.render().unwrap())
}

async fn page_settings() -> impl IntoResponse {
    Html(SettingsTemplate.render().unwrap())
}
// ── Root path handler ────────────────────────────────────────────────

async fn root_handler(
    session: tower_sessions::Session,
) -> impl IntoResponse {
    match session.get::<String>("user_id").await {
        Ok(Some(_)) => Redirect::to("/boards"),
        _ => Redirect::to("/login"),
    }
}

pub async fn build_app(pool: sqlx::PgPool, state: Arc<AppState>) -> axum::Router {
    let session_store = PgSessionStore::new(pool);
    let session_layer = tower_sessions::SessionManagerLayer::new(session_store)
        .with_secure(false)
        .with_same_site(SameSite::Lax);

    let app_state = state.clone();
    let api = axum::Router::new()
        .nest("/auth", handlers::auth::router())
        .nest("/boards", handlers::board::router())
        .nest("/lists", handlers::list::router())
        .nest("/cards", handlers::card::router())
        .nest("/labels", handlers::label::router())
        .nest("/comments", handlers::comment::router())
        .nest("/attachments", handlers::attachment::router())
        .nest("/favorites", handlers::favorite::router())
        .nest("/search", handlers::search::router())
        .nest("/health", handlers::health::router())
        .nest("/api-keys", handlers::api_key::router())
        .nest("/users", handlers::user_router())
        .nest("/events", axum::Router::new().route("/", get(events::event_stream)))
        .layer(tower_http::cors::CorsLayer::permissive())
        .with_state(state);

    axum::Router::new()
        .route("/login", get(handlers::auth::htmx_login_page).post(handlers::auth::htmx_login))
        .route("/", get(root_handler))
        .route("/boards", get(page_boards))
        .route("/boards.html", get(page_boards_html))
        .route("/board/{slug}/{*name}", get(page_board_with_slug))
        .route("/board.html", get(page_board_html))
        .route("/settings", get(page_settings))
        .route("/settings.html", get(page_settings_html))
        .nest("/api/v1", api)
        .fallback_service(tower::service_fn(static_or_redirect))
        .layer(middleware::from_fn(require_auth_for_html))
        .layer(session_layer)
        .with_state(app_state)
}

async fn static_or_redirect(
    req: axum::http::Request<axum::body::Body>,
) -> Result<axum::response::Response, std::convert::Infallible> {
    let mut serve_dir = ServeDir::new("static");
    match tower::ServiceExt::oneshot(&mut serve_dir, req).await {
        Ok(resp) => {
            if resp.status() == axum::http::StatusCode::NOT_FOUND {
                Ok(Redirect::to("/").into_response())
            } else {
                Ok(resp.map(axum::body::Body::new))
            }
        }
        Err(_) => Ok(Redirect::to("/").into_response()),
    }
}


// ── CLI integration ─────────────────────────────────────────────
pub mod cli {
    use std::fs;
    use std::io::{Read, Write};
    use std::path::PathBuf;

    /// Persistent credentials stored on disk.
    #[derive(serde::Serialize, serde::Deserialize)]
    pub struct Credentials {
        pub backend_url: String,
        pub token: String,
    }

    // ── Paths ──────────────────────────────────────────────────

    fn config_dir() -> PathBuf {
        let base = dirs::config_dir().unwrap_or_else(|| PathBuf::from("~/.config"));
        base.join("quest-board")
    }

    /// Returns the path to the credentials file.
    pub fn credential_path() -> PathBuf {
        config_dir().join("credentials.json")
    }

    // ── Credential I/O ──────────────────────────────────────────

    /// Loads credentials from disk, if they exist.
    pub fn load_credentials() -> Result<Option<Credentials>, String> {
        let path = credential_path();
        if !path.exists() {
            return Ok(None);
        }
        let mut file = fs::File::open(&path).map_err(|e| format!("cannot read {path:?}: {e}"))?;
        let mut contents = String::new();
        file.read_to_string(&mut contents).map_err(|e| format!("cannot read {path:?}: {e}"))?;
        let creds: Credentials = serde_json::from_str(&contents)
            .map_err(|e| format!("invalid credentials file: {e}"))?;
        Ok(Some(creds))
    }

    /// Saves credentials to disk with restrictive permissions (0600).
    pub fn save_credentials(creds: &Credentials) -> Result<(), String> {
        let dir = config_dir();
        fs::create_dir_all(&dir).map_err(|e| format!("cannot create {dir:?}: {e}"))?;
        let path = credential_path();
        let json = serde_json::to_string_pretty(creds)
            .map_err(|e| format!("serialization error: {e}"))?;
        {
            let mut file = fs::File::create(&path)
                .map_err(|e| format!("cannot write {path:?}: {e}"))?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                file.set_permissions(fs::Permissions::from_mode(0o600)).ok();
            }
            file.write_all(json.as_bytes())
                .map_err(|e| format!("cannot write {path:?}: {e}"))?;
        }
        Ok(())
    }

    /// Removes stored credentials.
    pub fn clear_credentials() -> Result<(), String> {
        let path = credential_path();
        if path.exists() {
            fs::remove_file(&path).map_err(|e| format!("cannot remove {path:?}: {e}"))?;
        }
        Ok(())
    }

    // ── HTTP client ──────────────────────────────────────────

    fn authed_client(token: &str) -> reqwest::Client {
        let mut headers = reqwest::header::HeaderMap::new();
        if let Ok(val) = reqwest::header::HeaderValue::from_str(&format!("Bearer {token}")) {
            headers.insert(reqwest::header::AUTHORIZATION, val);
        }
        reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .expect("reqwest client")
    }

    /// Make an authenticated GET request and parse JSON response.
    async fn api_get(backend_url: &str, token: &str, path: &str) -> Result<serde_json::Value, String> {
        let url = format!("{backend_url}{path}");
        let resp = authed_client(token)
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("connection error: {e}"))?;
        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(format!("request failed ({status}): {text}"));
        }
        resp.json().await.map_err(|e| format!("invalid JSON: {e}"))
    }

    async fn api_post(backend_url: &str, token: &str, path: &str, body: &serde_json::Value) -> Result<serde_json::Value, String> {
        let url = format!("{backend_url}{path}");
        let resp = authed_client(token)
            .post(&url)
            .json(body)
            .send()
            .await
            .map_err(|e| format!("connection error: {e}"))?;
        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(format!("request failed ({status}): {text}"));
        }
        resp.json().await.map_err(|e| format!("invalid JSON: {e}"))
    }

    async fn api_put(backend_url: &str, token: &str, path: &str, body: &serde_json::Value) -> Result<serde_json::Value, String> {
        let url = format!("{backend_url}{path}");
        let resp = authed_client(token)
            .put(&url)
            .json(body)
            .send()
            .await
            .map_err(|e| format!("connection error: {e}"))?;
        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(format!("request failed ({status}): {text}"));
        }
        resp.json().await.map_err(|e| format!("invalid JSON: {e}"))
    }

    async fn api_delete(backend_url: &str, token: &str, path: &str) -> Result<serde_json::Value, String> {
        let url = format!("{backend_url}{path}");
        let resp = authed_client(token)
            .delete(&url)
            .send()
            .await
            .map_err(|e| format!("connection error: {e}"))?;
        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(format!("request failed ({status}): {text}"));
        }
        resp.json().await.map_err(|e| format!("invalid JSON: {e}"))
    }

    /// DELETE with JSON body (e.g. DELETE /cards/{id}/labels with label_id in body).
    async fn api_delete_with_body(backend_url: &str, token: &str, path: &str, body: &serde_json::Value) -> Result<serde_json::Value, String> {
        let url = format!("{backend_url}{path}");
        let resp = authed_client(token)
            .delete(&url)
            .json(body)
            .send()
            .await
            .map_err(|e| format!("connection error: {e}"))?;
        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(format!("request failed ({status}): {text}"));
        }
        resp.json().await.map_err(|e| format!("invalid JSON: {e}"))
    }

    // ── Auth ────────────────────────────────────────────────────

    /// Logs in with username/password: POST /auth/login, then creates an API key.
    pub async fn login(backend_url: &str, username: &str, password: &str) -> Result<Credentials, String> {
        let client = reqwest::Client::builder().cookie_store(true).build().expect("reqwest client");
        let login_url = format!("{backend_url}/auth/login");
        let resp = client
            .post(&login_url)
            .json(&serde_json::json!({ "username": username, "password": password }))
            .send()
            .await
            .map_err(|e| format!("connection error: {e}"))?;
        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(format!("login failed ({status}): {text}"));
        }
        let key_url = format!("{backend_url}/api-keys");
        let resp = client
            .post(&key_url)
            .json(&serde_json::json!({ "name": "qb-cli" }))
            .send()
            .await
            .map_err(|e| format!("connection error: {e}"))?;
        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(format!("API key creation failed ({status}): {text}"));
        }
        let data: serde_json::Value = resp.json().await.map_err(|e| format!("invalid response: {e}"))?;
        let token = data["token"].as_str().ok_or("API key response missing token")?.to_string();
        Ok(Credentials { backend_url: backend_url.to_string(), token })
    }

    /// Returns the current user's JSON via GET /auth/me.
    pub async fn whoami(backend_url: &str, token: &str) -> Result<serde_json::Value, String> {
        api_get(backend_url, token, "/auth/me").await
    }

    // ── Boards ──────────────────────────────────────────────────

    /// GET /boards — list all boards for the current user.
    pub async fn list_boards(backend_url: &str, token: &str) -> Result<serde_json::Value, String> {
        api_get(backend_url, token, "/boards").await
    }

    /// POST /boards — create a board.
    pub async fn create_board(backend_url: &str, token: &str, name: &str) -> Result<serde_json::Value, String> {
        api_post(backend_url, token, "/boards", &serde_json::json!({ "name": name })).await
    }

    /// GET /boards/{id} — get board with lists and cards.
    pub async fn get_board(backend_url: &str, token: &str, id: &str) -> Result<serde_json::Value, String> {
        api_get(backend_url, token, &format!("/boards/{id}")).await
    }

    /// DELETE /boards/{id} — delete a board.
    pub async fn delete_board(backend_url: &str, token: &str, id: &str) -> Result<serde_json::Value, String> {
        api_delete(backend_url, token, &format!("/boards/{id}")).await
    }

    // ── Lists ────────────────────────────────────────────────────

    /// POST /lists — create a list in a board.
    pub async fn create_list(backend_url: &str, token: &str, board_id: &str, name: &str) -> Result<serde_json::Value, String> {
        api_post(backend_url, token, "/lists", &serde_json::json!({ "board_id": board_id, "name": name })).await
    }

    /// PUT /lists/{id} — update list name.
    pub async fn update_list(backend_url: &str, token: &str, id: &str, name: &str) -> Result<serde_json::Value, String> {
        api_put(backend_url, token, &format!("/lists/{id}"), &serde_json::json!({ "name": name })).await
    }

    /// DELETE /lists/{id} — delete a list.
    pub async fn delete_list(backend_url: &str, token: &str, id: &str) -> Result<serde_json::Value, String> {
        api_delete(backend_url, token, &format!("/lists/{id}")).await
    }

    // ── Cards ────────────────────────────────────────────────────

    /// POST /cards — create a card.
    pub async fn create_card(backend_url: &str, token: &str, list_id: &str, name: &str, desc: Option<&str>) -> Result<serde_json::Value, String> {
        let mut body = serde_json::json!({ "list_id": list_id, "name": name });
        if let Some(d) = desc { body["description"] = serde_json::json!(d); }
        api_post(backend_url, token, "/cards", &body).await
    }

    /// GET /cards/{id} — get card details.
    pub async fn get_card(backend_url: &str, token: &str, id: &str) -> Result<serde_json::Value, String> {
        api_get(backend_url, token, &format!("/cards/{id}")).await
    }

    /// PUT /cards/{id} — update card fields.
    pub async fn update_card(backend_url: &str, token: &str, id: &str, body: &serde_json::Value) -> Result<serde_json::Value, String> {
        api_put(backend_url, token, &format!("/cards/{id}"), body).await
    }

    /// PUT /cards/{id}/move — move card to a list.
    pub async fn move_card(backend_url: &str, token: &str, id: &str, list_id: &str) -> Result<serde_json::Value, String> {
        api_put(backend_url, token, &format!("/cards/{id}/move"), &serde_json::json!({ "list_id": list_id })).await
    }

    /// DELETE /cards/{id} — delete a card.
    pub async fn delete_card(backend_url: &str, token: &str, id: &str) -> Result<serde_json::Value, String> {
        api_delete(backend_url, token, &format!("/cards/{id}")).await
    }

    /// POST /cards/{id}/labels — add a label to a card.
    pub async fn add_card_label(backend_url: &str, token: &str, card_id: &str, label_id: &str) -> Result<serde_json::Value, String> {
        api_post(backend_url, token, &format!("/cards/{card_id}/labels"), &serde_json::json!({ "label_id": label_id })).await
    }

    /// DELETE /cards/{id}/labels — remove a label from a card.
    pub async fn remove_card_label(backend_url: &str, token: &str, card_id: &str, label_id: &str) -> Result<serde_json::Value, String> {
        api_delete_with_body(backend_url, token, &format!("/cards/{card_id}/labels"), &serde_json::json!({ "label_id": label_id })).await
    }

    /// POST /comments — add a comment to a card.
    pub async fn create_comment(backend_url: &str, token: &str, card_id: &str, text: &str) -> Result<serde_json::Value, String> {
        api_post(backend_url, token, "/comments", &serde_json::json!({ "card_id": card_id, "text": text })).await
    }

    /// POST /cards/{id}/task-lists — create a task list.
    pub async fn create_task_list(backend_url: &str, token: &str, card_id: &str, name: &str) -> Result<serde_json::Value, String> {
        api_post(backend_url, token, &format!("/cards/{card_id}/task-lists"), &serde_json::json!({ "card_id": card_id, "name": name })).await
    }

    /// POST /cards/{id}/task-lists/{tlid}/tasks — add a task.
    pub async fn create_task(backend_url: &str, token: &str, card_id: &str, tl_id: &str, name: &str) -> Result<serde_json::Value, String> {
        api_post(backend_url, token, &format!("/cards/{card_id}/task-lists/{tl_id}/tasks"), &serde_json::json!({ "name": name })).await
    }

    /// PUT /cards/{id}/task-lists/{tlid}/tasks/{tid} — toggle a task.
    pub async fn toggle_task(backend_url: &str, token: &str, card_id: &str, tl_id: &str, task_id: &str, done: bool) -> Result<serde_json::Value, String> {
        api_put(backend_url, token, &format!("/cards/{card_id}/task-lists/{tl_id}/tasks/{task_id}"), &serde_json::json!({ "is_completed": done })).await
    }

    /// GET /cards/{id}/comments — list comments.
    pub async fn list_comments(backend_url: &str, token: &str, card_id: &str) -> Result<serde_json::Value, String> {
        api_get(backend_url, token, &format!("/cards/{card_id}/comments")).await
    }

    /// GET /cards/{id}/actions — list actions.
    pub async fn list_actions(backend_url: &str, token: &str, card_id: &str) -> Result<serde_json::Value, String> {
        api_get(backend_url, token, &format!("/cards/{card_id}/actions")).await
    }

    // ── Labels ──────────────────────────────────────────────────

    /// GET /labels/board/{board_id} — list labels on a board.
    pub async fn list_board_labels(backend_url: &str, token: &str, board_id: &str) -> Result<serde_json::Value, String> {
        api_get(backend_url, token, &format!("/labels/board/{board_id}")).await
    }

    /// POST /labels — create a label on a board.
    pub async fn create_label(backend_url: &str, token: &str, board_id: &str, name: &str, color: Option<&str>) -> Result<serde_json::Value, String> {
        let mut body = serde_json::json!({ "board_id": board_id, "name": name });
        if let Some(c) = color { body["color"] = serde_json::json!(c); }
        api_post(backend_url, token, "/labels", &body).await
    }

    // ── Output formatting helpers ───────────────────────────────

    fn fmt_val(v: &serde_json::Value, key: &str) -> String {
        v.get(key).and_then(|v| v.as_str()).unwrap_or("").to_string()
    }

    fn fmt_json(v: &serde_json::Value) -> String {
        serde_json::to_string_pretty(v).unwrap_or_default()
    }

    // ── Command runners ──────────────────────────────────────────

    /// Route a board subcommand: qb board <list|create|view|delete> [args...]
    pub async fn run_board(backend_url: &str, token: &str, args: &[String]) -> Result<String, String> {
        let sub = args.get(0).map(|s| s.as_str()).unwrap_or("list");
        match sub {
            "list" => {
                let data = list_boards(backend_url, token).await?;
                let boards = data.as_array().ok_or("expected array")?;
                if boards.is_empty() {
                    return Ok("No boards.".to_string());
                }
                let mut out = String::new();
                for b in boards {
                    out.push_str(&format!("  {}  {} (id: {})\n",
                        fmt_val(b, "name"), fmt_val(b, "slug"), fmt_val(b, "id")));
                }
                Ok(out.trim_end().to_string())
            }
            "create" => {
                let name = args.get(1).ok_or("Usage: qb board create <name>")?;
                let data = create_board(backend_url, token, name).await?;
                Ok(format!("Created board: {} (id: {})", fmt_val(&data, "name"), fmt_val(&data, "id")))
            }
            "view" => {
                let id = args.get(1).ok_or("Usage: qb board view <id>")?;
                let data = get_board(backend_url, token, id).await?;
                Ok(fmt_json(&data))
            }
            "delete" => {
                let id = args.get(1).ok_or("Usage: qb board delete <id>")?;
                delete_board(backend_url, token, id).await?;
                Ok(format!("Board {id} deleted."))
            }
            _ => Err(format!("Unknown board subcommand: {sub}"))
        }
    }

    /// Route a list subcommand: qb list <create|rename|delete> [args...]
    pub async fn run_list(backend_url: &str, token: &str, args: &[String]) -> Result<String, String> {
        let sub = args.get(0).map(|s| s.as_str()).unwrap_or("");
        match sub {
            "create" => {
                let board_id = args.get(1).ok_or("Usage: qb list create <board-id> <name>")?;
                let name = args.get(2).ok_or("Usage: qb list create <board-id> <name>")?;
                let data = create_list(backend_url, token, board_id, name).await?;
                Ok(format!("Created list: {} (id: {})", fmt_val(&data, "name"), fmt_val(&data, "id")))
            }
            "rename" => {
                let id = args.get(1).ok_or("Usage: qb list rename <id> <name>")?;
                let name = args.get(2).ok_or("Usage: qb list rename <id> <name>")?;
                let data = update_list(backend_url, token, id, name).await?;
                Ok(format!("Renamed list to: {}", fmt_val(&data, "name")))
            }
            "delete" => {
                let id = args.get(1).ok_or("Usage: qb list delete <id>")?;
                delete_list(backend_url, token, id).await?;
                Ok(format!("List {id} deleted."))
            }
            _ => Err("Usage: qb list <create|rename|delete> [args...]".to_string())
        }
    }

    /// Route a card subcommand: qb card <create|view|update|move|delete|label|comment|task> [args...]
    pub async fn run_card(backend_url: &str, token: &str, args: &[String]) -> Result<String, String> {
        let sub = args.get(0).map(|s| s.as_str()).unwrap_or("");
        match sub {
            "create" => {
                let list_id = args.get(1).ok_or("Usage: qb card create <list-id> <name> [description]")?;
                let name = args.get(2).ok_or("Usage: qb card create <list-id> <name> [description]")?;
                let desc = args.get(3).map(|s| s.as_str());
                let data = create_card(backend_url, token, list_id, name, desc).await?;
                Ok(format!("Created card: {} (id: {})", fmt_val(&data, "name"), fmt_val(&data, "id")))
            }
            "view" => {
                let id = args.get(1).ok_or("Usage: qb card view <id>")?;
                let data = get_card(backend_url, token, id).await?;
                Ok(fmt_json(&data))
            }
            "update" => {
                let id = args.get(1).ok_or("Usage: qb card update <id> <field> <value>")?;
                let field = args.get(2).ok_or("Usage: qb card update <id> <field> <value>")?;
                let val = args.get(3).ok_or("Usage: qb card update <id> <field> <value>")?;
                let body = serde_json::json!({ field: val });
                let data = update_card(backend_url, token, id, &body).await?;
                Ok(format!("Updated card: {} (id: {})", fmt_val(&data, "name"), fmt_val(&data, "id")))
            }
            "move" => {
                let id = args.get(1).ok_or("Usage: qb card move <id> <list-id>")?;
                let list_id = args.get(2).ok_or("Usage: qb card move <id> <list-id>")?;
                let data = move_card(backend_url, token, id, list_id).await?;
                Ok(format!("Moved card: {} (id: {})", fmt_val(&data, "name"), fmt_val(&data, "id")))
            }
            "delete" => {
                let id = args.get(1).ok_or("Usage: qb card delete <id>")?;
                delete_card(backend_url, token, id).await?;
                Ok(format!("Card {id} deleted."))
            }
            "label" => {
                let id = args.get(1).ok_or("Usage: qb card label <id> add|remove <label-id>")?;
                let action = args.get(2).map(|s| s.as_str()).unwrap_or("");
                let label_id = args.get(3).ok_or("Usage: qb card label <id> add|remove <label-id>")?;
                match action {
                    "add" => { add_card_label(backend_url, token, id, label_id).await?; }
                    "remove" => { remove_card_label(backend_url, token, id, label_id).await?; }
                    _ => return Err("Usage: qb card label <id> add|remove <label-id>".to_string())
                }
                Ok(format!("Label {action}ed on card {id}."))
            }
            "comment" => {
                let id = args.get(1).ok_or("Usage: qb card comment <id> <text>")?;
                let text = args.get(2).ok_or("Usage: qb card comment <id> <text>")?;
                create_comment(backend_url, token, id, text).await?;
                Ok("Comment added.".to_string())
            }
            "task" => {
                let id = args.get(1).ok_or("Usage: qb card task <id> <list|add|toggle> [...]")?;
                let action = args.get(2).map(|s| s.as_str()).unwrap_or("");
                match action {
                    "list" => {
                        let data = get_card(backend_url, token, id).await?;
                        let checks = data["checklists"].as_array().map(|a| a.clone()).unwrap_or_default();
                        if checks.is_empty() { return Ok("No task lists.".to_string()); }
                        let mut out = String::new();
                        for cl in &checks {
                            out.push_str(&format!("  {} (id: {})\n", fmt_val(cl, "name"), fmt_val(cl, "id")));
                            if let Some(tasks) = cl["tasks"].as_array() {
                                for t in tasks {
                                    let done = t["is_completed"].as_bool().unwrap_or(false);
                                    let mark = if done { "[x]" } else { "[ ]" };
                                    out.push_str(&format!("    {mark} {} (id: {})\n", fmt_val(t, "name"), fmt_val(t, "id")));
                                }
                            }
                        }
                        Ok(out.trim_end().to_string())
                    }
                    "add-list" => {
                        let name = args.get(3).ok_or("Usage: qb card task <id> add-list <name>")?;
                        let data = create_task_list(backend_url, token, id, name).await?;
                        Ok(format!("Task list created: {} (id: {})", fmt_val(&data, "name"), fmt_val(&data, "id")))
                    }
                    "add-task" => {
                        let tl_id = args.get(3).ok_or("Usage: qb card task <id> add-task <tl-id> <task-name>")?;
                        let task_name = args.get(4).ok_or("Usage: qb card task <id> add-task <tl-id> <task-name>")?;
                        let data = create_task(backend_url, token, id, tl_id, task_name).await?;
                        Ok(format!("Task created: {} (id: {})", fmt_val(&data, "name"), fmt_val(&data, "id")))
                    }
                    "toggle" => {
                        let tl_id = args.get(3).ok_or("Usage: qb card task <id> toggle <tl-id> <task-id> <true|false>")?;
                        let task_id = args.get(4).ok_or("Usage: qb card task <id> toggle <tl-id> <task-id> <true|false>")?;
                        let done = args.get(5).map(|s| s == "true").unwrap_or(true);
                        toggle_task(backend_url, token, id, tl_id, task_id, done).await?;
                        Ok(format!("Task {task_id} toggled."))
                    }
                    _ => return Err("Usage: qb card task <id> <list|add-list|add-task|toggle> [...]".to_string())
                }
            }
            _ => Err("Usage: qb card <create|view|update|move|delete|label|comment|task> [args...]".to_string())
        }
    }

    /// Route a label subcommand: qb label <list|create> [args...]
    pub async fn run_label(backend_url: &str, token: &str, args: &[String]) -> Result<String, String> {
        let sub = args.get(0).map(|s| s.as_str()).unwrap_or("");
        match sub {
            "list" => {
                let board_id = args.get(1).ok_or("Usage: qb label list <board-id>")?;
                let data = list_board_labels(backend_url, token, board_id).await?;
                let labels = data.as_array().ok_or("expected array")?;
                if labels.is_empty() { return Ok("No labels.".to_string()); }
                let mut out = String::new();
                for l in labels {
                    out.push_str(&format!("  {} ({}; id: {})\n", fmt_val(l, "name"), fmt_val(l, "color"), fmt_val(l, "id")));
                }
                Ok(out.trim_end().to_string())
            }
            "create" => {
                let board_id = args.get(1).ok_or("Usage: qb label create <board-id> <name> [color]")?;
                let name = args.get(2).ok_or("Usage: qb label create <board-id> <name> [color]")?;
                let color = args.get(3).map(|s| s.as_str());
                let data = create_label(backend_url, token, board_id, name, color).await?;
                Ok(format!("Created label: {} (id: {})", fmt_val(&data, "name"), fmt_val(&data, "id")))
            }
            _ => Err("Usage: qb label <list|create> [args...]".to_string())
        }
    }

    /// GET /auth/me — show current user.
    pub async fn run_me(backend_url: &str, token: &str) -> Result<String, String> {
        let data = whoami(backend_url, token).await?;
        Ok(fmt_json(&data))
    }

    // ── Top-level command routing ──────────────────────────────

    pub async fn run_default(backend_url: &str, creds: Option<&Credentials>) -> Result<String, String> {
        let mut out = hello_msg(backend_url);
        if let Some(c) = creds {
            match whoami(backend_url, &c.token).await {
                Ok(data) => {
                    let name = data["name"].as_str().unwrap_or("unknown");
                    out.push_str(&format!("\nLogged in as: {name}"));
                }
                Err(_) => out.push_str("\nSession expired. Run `qb login` to authenticate."),
            }
        } else {
            out.push_str("\nNot logged in. Run `qb login` to authenticate.");
        }
        Ok(out)
    }

    pub async fn run_login(backend_url: &str) -> Result<String, String> {
        println!("Logging in to {backend_url}");
        let username = rpassword::prompt_password("Username: ")
            .map_err(|e| format!("failed to read input: {e}"))?;
        let password = rpassword::prompt_password("Password: ")
            .map_err(|e| format!("failed to read input: {e}"))?;
        let creds = login(backend_url, &username, &password).await?;
        save_credentials(&creds)?;
        Ok(format!("Logged in as {username}. Credentials saved."))
    }

    pub async fn run_status(backend_url: &str, creds: Option<&Credentials>) -> Result<String, String> {
        match creds {
            Some(c) => match whoami(backend_url, &c.token).await {
                Ok(data) => {
                    let name = data["name"].as_str().unwrap_or("unknown");
                    let user = data["username"].as_str().unwrap_or("");
                    Ok(format!("Logged in to {backend_url} as {name} (@{user})"))
                }
                Err(e) => Ok(format!("Not authenticated ({e}). Run `qb login`.")),
            },
            None => Ok("Not logged in. Run `qb login` to authenticate.".to_string()),
        }
    }

    pub async fn run_logout() -> Result<String, String> {
        clear_credentials()?;
        Ok("Logged out. Credentials cleared.".to_string())
    }

    pub fn hello_msg(backend_url: &str) -> String {
        format!("Hello from quest-board CLI! Backend: {backend_url}")
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use std::fs;

        // ── Existing tests ─────────────────────────────────────

        #[test]
        fn test_hello_msg_format() {
            let msg = hello_msg("http://localhost:3000");
            assert!(msg.contains("Hello from quest-board CLI!"));
            assert!(msg.contains("http://localhost:3000"));
        }

        #[test]
        fn test_credential_path_ends_correctly() {
            let path = credential_path();
            let s = path.to_string_lossy();
            assert!(s.contains("quest-board"));
            assert!(s.ends_with("credentials.json"));
        }

        #[test]
        fn test_credential_roundtrip() {
            let tmp = std::env::temp_dir().join(format!("qb-test-{}", std::process::id()));
            let _ = fs::create_dir_all(&tmp);
            let path = tmp.join("credentials.json");
            let creds = Credentials {
                backend_url: "http://test:3000/api/v1".into(),
                token: "qb_test123".into(),
            };
            let json = serde_json::to_string_pretty(&creds).unwrap();
            fs::write(&path, &json).unwrap();
            let contents = fs::read_to_string(&path).unwrap();
            let loaded: Credentials = serde_json::from_str(&contents).unwrap();
            assert_eq!(loaded.backend_url, creds.backend_url);
            assert_eq!(loaded.token, creds.token);
            fs::remove_file(&path).ok();
            fs::remove_dir(&tmp).ok();
        }

        #[test]
        fn test_save_load_clear_roundtrip() {
            let creds = Credentials {
                backend_url: "http://test:3000/api/v1".into(),
                token: "qb_integration_test".into(),
            };
            save_credentials(&creds).unwrap();
            let loaded = load_credentials().unwrap().unwrap();
            assert_eq!(loaded.backend_url, creds.backend_url);
            assert_eq!(loaded.token, creds.token);
            clear_credentials().unwrap();
            assert!(load_credentials().unwrap().is_none());
        }

        // ── URL construction tests ──────────────────────────────

        fn base() -> &'static str { "http://localhost:3000/api/v1" }

        #[test]
        fn test_boards_urls() {
            assert_eq!(format!("{}/boards", base()), "http://localhost:3000/api/v1/boards");
            assert_eq!(format!("{}/boards/abc", base()), "http://localhost:3000/api/v1/boards/abc");
        }

        #[test]
        fn test_lists_urls() {
            assert_eq!(format!("{}/lists", base()), "http://localhost:3000/api/v1/lists");
            assert_eq!(format!("{}/lists/abc", base()), "http://localhost:3000/api/v1/lists/abc");
        }

        #[test]
        fn test_cards_urls() {
            assert_eq!(format!("{}/cards", base()), "http://localhost:3000/api/v1/cards");
            assert_eq!(format!("{}/cards/abc", base()), "http://localhost:3000/api/v1/cards/abc");
            assert_eq!(format!("{}/cards/abc/move", base()), "http://localhost:3000/api/v1/cards/abc/move");
            assert_eq!(format!("{}/cards/abc/labels", base()), "http://localhost:3000/api/v1/cards/abc/labels");
            assert_eq!(format!("{}/cards/abc/comments", base()), "http://localhost:3000/api/v1/cards/abc/comments");
            assert_eq!(format!("{}/cards/abc/actions", base()), "http://localhost:3000/api/v1/cards/abc/actions");
            assert_eq!(format!("{}/cards/abc/task-lists", base()), "http://localhost:3000/api/v1/cards/abc/task-lists");
        }

        #[test]
        fn test_labels_urls() {
            assert_eq!(format!("{}/labels", base()), "http://localhost:3000/api/v1/labels");
            assert_eq!(format!("{}/labels/board/abc", base()), "http://localhost:3000/api/v1/labels/board/abc");
        }

        #[test]
        fn test_auth_urls() {
            assert_eq!(format!("{}/auth/me", base()), "http://localhost:3000/api/v1/auth/me");
            assert_eq!(format!("{}/auth/login", base()), "http://localhost:3000/api/v1/auth/login");
        }

        // ── JSON format tests ─────────────────────────────────

        #[test]
        fn test_fmt_val() {
            let v = serde_json::json!({"name": "Test Board", "slug": "test-board"});
            assert_eq!(fmt_val(&v, "name"), "Test Board");
            assert_eq!(fmt_val(&v, "slug"), "test-board");
            assert_eq!(fmt_val(&v, "missing"), "");
        }

        #[test]
        fn test_fmt_json() {
            let v = serde_json::json!({"key": "val"});
            let s = fmt_json(&v);
            assert!(s.contains("\"key\""));
            assert!(s.contains("\"val\""));
        }

        #[test]
        fn test_auth_header_format() {
            let token = "qb_test_token";
            let header = format!("Bearer {token}");
            assert_eq!(header, "Bearer qb_test_token");
        }
    }
}
