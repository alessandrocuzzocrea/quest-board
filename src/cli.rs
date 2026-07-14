use std::env;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    let backend_url = env::var("QB_URL")
        .unwrap_or_else(|_| "http://localhost:3000/api/v1".into());

    let subcommand = args.get(1).map(|s| s.as_str()).unwrap_or("default");

    let result: Result<String, String> = match subcommand {
        "login" => quest_board::cli::run_login(&backend_url).await,
        "logout" => quest_board::cli::run_logout().await,
        "status" => {
            let creds = quest_board::cli::load_credentials().unwrap_or(None);
            quest_board::cli::run_status(&backend_url, creds.as_ref()).await
        }
        _ => {
            let creds = quest_board::cli::load_credentials().unwrap_or(None);
            quest_board::cli::run_default(&backend_url, creds.as_ref()).await
        }
    };

    match result {
        Ok(output) => println!("{output}"),
        Err(e) => eprintln!("Error: {e}"),
    }
}
