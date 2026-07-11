use utoipa::OpenApi;
fn main() {
    let doc = quest_board::api_doc::ApiDoc::openapi();
    let json = serde_json::to_string_pretty(&doc).expect("failed to serialize OpenAPI spec");
    let out = std::env::args().nth(1).unwrap_or_else(|| "../frontend/openapi.json".into());
    std::fs::write(&out, &json).expect("failed to write OpenAPI spec");
    eprintln!("OpenAPI spec written to {out}");
}
