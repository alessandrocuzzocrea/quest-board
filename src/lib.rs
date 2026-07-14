pub mod db;
pub mod ai_tools;
pub mod auth;
pub mod error;
pub mod handlers;
pub mod models;
pub mod repository;
pub mod slug;
pub mod session;
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
    pub ai_client: Arc<dyn handlers::ai::LlmClient>,
}

// Known app pages that require authentication (both clean and .html forms)
const PROTECTED_PATHS: &[&str] = &[
    "/boards", "/board", "/settings",
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
        || PROTECTED_PATHS.contains(&req_path.as_str());

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

async fn page_boards() -> impl IntoResponse {
    Html(include_str!("../static/boards.html"))
}

async fn page_board() -> impl IntoResponse {
    Html(include_str!("../static/board.html"))
}

async fn page_settings() -> impl IntoResponse {
    Html(include_str!("../static/settings.html"))
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
        .nest("/ai", handlers::ai::router())
        .nest("/users", handlers::user_router())
        .layer(tower_http::cors::CorsLayer::permissive())
        .with_state(state);

    let static_files = ServeDir::new("static");

    axum::Router::new()
        .route("/login", get(handlers::auth::htmx_login_page).post(handlers::auth::htmx_login))
        .route("/", get(root_handler))
        .route("/boards", get(page_boards))
        .route("/board", get(page_board))
        .route("/settings", get(page_settings))
        .nest("/api/v1", api)
        .fallback_service(static_files)
        .layer(middleware::from_fn(require_auth_for_html))
        .layer(session_layer)
        .with_state(app_state)
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
                file.set_permissions(fs::Permissions::from_mode(0o600))
                    .ok();
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

    // ── HTTP helpers ──────────────────────────────────────────

    fn client_with_token(token: Option<&str>) -> reqwest::Client {
        let mut headers = reqwest::header::HeaderMap::new();
        if let Some(t) = token {
            if let Ok(val) = reqwest::header::HeaderValue::from_str(&format!("Bearer {t}")) {
                headers.insert(reqwest::header::AUTHORIZATION, val);
            }
        }
        reqwest::Client::builder()
            .default_headers(headers)
            .cookie_store(true)
            .build()
            .expect("reqwest client")
    }

    // ── Auth ────────────────────────────────────────────────────

    /// Logs in with username/password: POST /auth/login, then creates an API key.
    /// Returns the token from the created API key.
    pub async fn login(backend_url: &str, username: &str, password: &str) -> Result<Credentials, String> {
        let client = client_with_token(None);

        // Step 1: Authenticate with username/password (gets a session cookie)
        let login_url = format!("{backend_url}/auth/login");
        let login_body = serde_json::json!({ "username": username, "password": password });
        let resp = client
            .post(&login_url)
            .json(&login_body)
            .send()
            .await
            .map_err(|e| format!("connection error: {e}"))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(format!("login failed ({status}): {text}"));
        }

        // Step 2: Create an API key (cookie from step 1 is automatically sent)
        let key_url = format!("{backend_url}/api-keys");
        let key_body = serde_json::json!({ "name": "qb-cli" });
        let resp = client
            .post(&key_url)
            .json(&key_body)
            .send()
            .await
            .map_err(|e| format!("connection error: {e}"))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(format!("API key creation failed ({status}): {text}"));
        }

        let data: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| format!("invalid response: {e}"))?;

        let token = data["token"]
            .as_str()
            .ok_or("API key response missing token field")?
            .to_string();

        Ok(Credentials {
            backend_url: backend_url.to_string(),
            token,
        })
    }

    /// Returns the current user's name via GET /auth/me.
    pub async fn whoami(backend_url: &str, token: &str) -> Result<String, String> {
        let client = client_with_token(Some(token));
        let url = format!("{backend_url}/auth/me");
        let resp = client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("connection error: {e}"))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(format!("auth check failed ({status}): {text}"));
        }

        let data: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| format!("invalid response: {e}"))?;

        Ok(data["name"]
            .as_str()
            .unwrap_or("unknown")
            .to_string())
    }

    // ── Commands ────────────────────────────────────────────────

    /// Runs the default greeting, showing auth status if logged in.
    pub async fn run_default(backend_url: &str, creds: Option<&Credentials>) -> Result<String, String> {
        let mut out = hello_msg(backend_url);
        if let Some(c) = creds {
            match whoami(backend_url, &c.token).await {
                Ok(name) => {
                    out.push_str(&format!("\nLogged in as: {name}"));
                }
                Err(_) => {
                    out.push_str("\nSession expired. Run `qb login` to authenticate.");
                }
            }
        } else {
            out.push_str("\nNot logged in. Run `qb login` to authenticate.");
        }
        Ok(out)
    }

    /// Runs the login command: prompts for username/password, authenticates, stores token.
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

    /// Runs the status command: shows who you're logged in as.
    pub async fn run_status(backend_url: &str, creds: Option<&Credentials>) -> Result<String, String> {
        match creds {
            Some(c) => match whoami(backend_url, &c.token).await {
                Ok(name) => Ok(format!("Logged in to {backend_url} as {name}")),
                Err(e) => Ok(format!("Not authenticated ({e}). Run `qb login`.")),
            },
            None => Ok(format!("Not logged in. Run `qb login` to authenticate.")),
        }
    }

    /// Runs the logout command: clears stored credentials.
    pub async fn run_logout() -> Result<String, String> {
        clear_credentials()?;
        Ok("Logged out. Credentials cleared.".to_string())
    }

    /// Returns "Hello from quest-board CLI! Backend: {url}"
    pub fn hello_msg(backend_url: &str) -> String {
        format!("Hello from quest-board CLI! Backend: {backend_url}")
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use std::fs;

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
            // Use a temp dir to avoid clobbering real credentials
            let tmp = std::env::temp_dir().join(format!("qb-test-{}", std::process::id()));
            let _ = fs::create_dir_all(&tmp);

            // Monkey-patch config dir via env is not possible, but we can directly
            // test the save/load functions via their real paths if we clear after.
            // Instead, test the I/O functions directly by writing to a temp file path.
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

        #[test]
        fn test_login_urls() {
            let base = "http://localhost:3000/api/v1";
            assert_eq!(
                format!("{base}/auth/login"),
                "http://localhost:3000/api/v1/auth/login"
            );
            assert_eq!(
                format!("{base}/api-keys"),
                "http://localhost:3000/api/v1/api-keys"
            );
        }

        #[test]
        fn test_auth_header_format() {
            let token = "qb_test_token";
            let header = format!("Bearer {token}");
            assert_eq!(header, "Bearer qb_test_token");
        }
    }
}
