const USAGE: &str = "\
Usage: qb [COMMAND]

Commands:
  login     Authenticate with the backend and save an API token
  logout    Clear stored credentials
  status    Show current authentication status
  help      Print this help message

Environment:
  QB_URL    Backend API base URL (default: http://localhost:3000/api/v1)

Examples:
  qb                  Show greeting and auth status
  qb login            Log in with username/password
  qb status           Check who you're logged in as
  qb logout           Clear credentials
";

fn print_help() {
    print!("{USAGE}");
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    let backend_url = std::env::var("QB_URL")
        .unwrap_or_else(|_| "http://localhost:3000/api/v1".into());

    let subcommand = args.get(1).map(|s| s.as_str()).unwrap_or("default");

    match subcommand {
        "help" | "--help" | "-h" => {
            print_help();
            return;
        }
        _ => {}
    }

    let result: Result<String, String> = match subcommand {
        "login" => quest_board::cli::run_login(&backend_url).await,
        "logout" => quest_board::cli::run_logout().await,
        "status" => {
            let creds = quest_board::cli::load_credentials().unwrap_or(None);
            quest_board::cli::run_status(&backend_url, creds.as_ref()).await
        }
        // Any unrecognized command shows help
        other => {
            if other != "default" {
                eprintln!("Unknown command: {other}\n");
            }
            print_help();
            return;
        }
    };

    match result {
        Ok(output) => println!("{output}"),
        Err(e) => eprintln!("Error: {e}"),
    }
}
