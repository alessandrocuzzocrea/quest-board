const USAGE: &str = "\
Usage: qb [COMMAND]

Authentication:
  login                 Authenticate with username/password
  logout                Clear stored credentials
  status                Show current authentication status

Resources:
  board list            List all boards
  board create <name>   Create a new board
  board view <id>       View board details and lists
  board delete <id>     Delete a board

  list create <board-id> <name>   Create a list in a board
  list rename <id> <name>         Rename a list
  list delete <id>                Delete a list

  card create <list-id> <name> [desc]   Create a card
  card view <id>                         View card details
  card update <id> <field> <value>      Update a card field
  card move <id> <list-id>              Move card to another list
  card delete <id>                      Delete a card
  card label <id> add|remove <label-id> Toggle a label on a card
  card comment <id> <text>              Add a comment
  card task <id> list                   List task lists and tasks
  card task <id> add-list <name>        Add a task list
  card task <id> add-task <tl-id> <task-name>  Add a task
  card task <id> toggle <tl-id> <task-id> [true|false]  Toggle a task

  label list <board-id>        List labels on a board
  label create <board-id> <name> [color]  Create a label

  me                        Show current user info

  help                      Print this help message

Environment:
  QB_URL    Backend API base URL (default: http://localhost:3000/api/v1)

Examples:
  qb login
  qb board list
  qb board create 'My Board'
  qb card create <list-id> 'Fix login bug' 'need to fix auth'
  qb card task <id> list
";

fn print_help() {
    print!("{USAGE}");
}

fn require_creds() -> quest_board::cli::Credentials {
    quest_board::cli::load_credentials()
        .unwrap_or(None)
        .expect("Not logged in. Run `qb login` first.")
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    let backend_url = std::env::var("QB_URL")
        .unwrap_or_else(|_| "http://localhost:3000/api/v1".into());

    let subcommand = args.get(1).map(|s| s.as_str()).unwrap_or("default");

    match subcommand {
        "help" | "--help" | "-h" => { print_help(); return; }
        _ => {}
    }

    let result: Result<String, String> = match subcommand {
        "login" => quest_board::cli::run_login(&backend_url).await,
        "logout" => quest_board::cli::run_logout().await,
        "status" => {
            let creds = quest_board::cli::load_credentials().unwrap_or(None);
            quest_board::cli::run_status(&backend_url, creds.as_ref()).await
        }
        "me" => {
            let c = require_creds();
            quest_board::cli::run_me(&backend_url, &c.token).await
        }
        "board" => {
            let c = require_creds();
            let subargs: Vec<String> = args.iter().skip(2).cloned().collect();
            quest_board::cli::run_board(&backend_url, &c.token, &subargs).await
        }
        "list" => {
            let c = require_creds();
            let subargs: Vec<String> = args.iter().skip(2).cloned().collect();
            quest_board::cli::run_list(&backend_url, &c.token, &subargs).await
        }
        "card" => {
            let c = require_creds();
            let subargs: Vec<String> = args.iter().skip(2).cloned().collect();
            quest_board::cli::run_card(&backend_url, &c.token, &subargs).await
        }
        "label" => {
            let c = require_creds();
            let subargs: Vec<String> = args.iter().skip(2).cloned().collect();
            quest_board::cli::run_label(&backend_url, &c.token, &subargs).await
        }
        // Unknown command → show help; no args → show greeting
        other => {
            if other != "default" {
                eprintln!("Unknown command: {other}\n");
                print_help();
                return;
            }
            let creds = quest_board::cli::load_credentials().unwrap_or(None);
            quest_board::cli::run_default(&backend_url, creds.as_ref()).await
        }
    };

    match result {
        Ok(output) => println!("{output}"),
        Err(e) => eprintln!("Error: {e}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_usage_contains_commands() {
        assert!(USAGE.contains("login"));
        assert!(USAGE.contains("logout"));
        assert!(USAGE.contains("status"));
        assert!(USAGE.contains("board"));
        assert!(USAGE.contains("list"));
        assert!(USAGE.contains("card"));
        assert!(USAGE.contains("label"));
        assert!(USAGE.contains("me"));
        assert!(USAGE.contains("help"));
        assert!(USAGE.contains("QB_URL"));
    }
}
