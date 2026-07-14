#[tokio::main]
async fn main() {
    let backend_url = std::env::var("QB_URL")
        .unwrap_or_else(|_| "http://localhost:3000/api/v1".into());

    match quest_board::cli::run(&backend_url).await {
        Ok(output) => println!("{output}"),
        Err(e) => eprintln!("Error: {e}"),
    }
}
