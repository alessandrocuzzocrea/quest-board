use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

pub struct Tool {
    pub name: &'static str,
    pub description: &'static str,
    pub parameters: serde_json::Value,
}

pub fn all_tools() -> Vec<Tool> {
    vec![
        Tool {
            name: "create_card",
            description: "Create a new card in a list",
            parameters: json!({
                "type": "object",
                "properties": {
                    "list_id": {"type": "string", "description": "ID of the list"},
                    "name": {"type": "string", "description": "Card name"},
                    "description": {"type": "string", "description": "Card description"}
                },
                "required": ["list_id", "name"]
            }),
        },
        Tool {
            name: "search_cards",
            description: "Search cards and boards by query",
            parameters: json!({
                "type": "object",
                "properties": {
                    "query": {"type": "string", "description": "Search query"}
                },
                "required": ["query"]
            }),
        },
        Tool {
            name: "get_board_lists",
            description: "Get all lists in a board",
            parameters: json!({
                "type": "object",
                "properties": {
                    "board_id": {"type": "string", "description": "Board ID"}
                },
                "required": ["board_id"]
            }),
        },
    ]
}

#[derive(Debug)]
pub struct ToolResult {
    pub success: bool,
    pub data: serde_json::Value,
}

pub async fn execute_tool(name: &str, args: &serde_json::Value, user_id: &Uuid, pool: &PgPool) -> ToolResult {
    match name {
        "create_card" => execute_create_card(args, pool, user_id).await,
        "search_cards" => execute_search(args, user_id, pool).await,
        "get_board_lists" => execute_get_board_lists(args, pool).await,
        _ => ToolResult { success: false, data: json!({"error": format!("Unknown tool: {name}")}) },
    }
}

async fn execute_create_card(args: &serde_json::Value, pool: &PgPool, user_id: &Uuid) -> ToolResult {
    let list_id = match args["list_id"].as_str() {
        Some(v) => v,
        None => return ToolResult { success: false, data: json!({"error": "list_id required"}) },
    };
    let name = match args["name"].as_str() {
        Some(v) => v,
        None => return ToolResult { success: false, data: json!({"error": "name required"}) },
    };
    let parsed = match Uuid::parse_str(list_id) {
        Ok(v) => v,
        Err(_) => return ToolResult { success: false, data: json!({"error": "invalid list_id"}) },
    };

    let exists: Option<(String,)> = sqlx::query_as("SELECT board_id::text FROM lists WHERE id = $1")
        .bind(parsed).fetch_optional(pool).await.unwrap_or(None);
    let board_id = match exists {
        Some((bid,)) => bid,
        None => return ToolResult { success: false, data: json!({"error": "List not found"}) },
    };

    let card_id = Uuid::new_v4();
    let now = chrono::Utc::now().to_rfc3339();
    let desc = args["description"].as_str().unwrap_or("");

    let result = sqlx::query(
        "INSERT INTO cards (id, board_id, list_id, name, description, created_by, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
    )
    .bind(card_id)
    .bind(Uuid::parse_str(&board_id).unwrap_or_default())
    .bind(parsed)
    .bind(name)
    .bind(desc)
    .bind(user_id)
    .bind(&now)
    .bind(&now)
    .execute(pool).await;

    match result {
        Ok(_) => ToolResult { success: true, data: json!({"card_id": card_id.to_string(), "name": name, "board_id": board_id}) },
        Err(e) => ToolResult { success: false, data: json!({"error": format!("Failed to create card: {e}")}) },
    }
}

async fn execute_search(args: &serde_json::Value, user_id: &Uuid, pool: &PgPool) -> ToolResult {
    let query = args["query"].as_str().unwrap_or("");
    let pattern = format!("%{}%", query);

    let cards: Vec<(String, String)> = sqlx::query_as(
        "SELECT c.id, c.name FROM cards c JOIN lists l ON c.list_id = l.id JOIN boards b ON c.board_id = b.id LEFT JOIN board_members bm ON b.id = bm.board_id AND bm.user_id = $1 WHERE (c.name ILIKE $2 OR c.description ILIKE $3) AND (b.created_by = $4 OR bm.user_id = $5) LIMIT 10",
    ).bind(user_id).bind(&pattern).bind(&pattern).bind(user_id).bind(user_id)
    .fetch_all(pool).await.unwrap_or_default();
    let boards: Vec<(String, String)> = sqlx::query_as(
        "SELECT id, name FROM boards WHERE name ILIKE $1 AND created_by = $2 LIMIT 5",
    ).bind(&pattern).bind(user_id)
    .fetch_all(pool).await.unwrap_or_default();

    ToolResult {
        success: true,
        data: json!({
            "cards": cards.into_iter().map(|(id, name)| json!({"id": id, "name": name})).collect::<Vec<_>>(),
            "boards": boards.into_iter().map(|(id, name)| json!({"id": id, "name": name})).collect::<Vec<_>>(),
        }),
    }
}

async fn execute_get_board_lists(args: &serde_json::Value, pool: &PgPool) -> ToolResult {
    let board_id = match args["board_id"].as_str() {
        Some(v) => v,
        None => return ToolResult { success: false, data: json!({"error": "board_id required"}) },
    };
    let parsed = match Uuid::parse_str(board_id) {
        Ok(v) => v,
        Err(_) => return ToolResult { success: false, data: json!({"error": "invalid board_id"}) },
    };

    let lists: Vec<(String, Option<String>)> = sqlx::query_as(
        "SELECT id::text, name FROM lists WHERE board_id = $1 ORDER BY position",
    ).bind(parsed).fetch_all(pool).await.unwrap_or_default();

    ToolResult {
        success: true,
        data: json!({ "lists": lists.into_iter().map(|(id, name)| json!({"id": id, "name": name})).collect::<Vec<_>>() }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::Row;

    async fn setup_pool() -> (sqlx::PgPool, Uuid) {
        dotenvy::from_filename("../backend/.env.test").ok();
        let db = std::env::var("DATABASE_URL").unwrap_or_else(|_| "postgres://postgres:quest@localhost:5432/quest_test".into());
        let pool = sqlx::PgPool::connect(&db).await.unwrap();
        sqlx::query("DROP TABLE IF EXISTS api_keys,sessions,favorites,notifications,actions,tasks,task_lists,attachments,comments,card_labels,labels,card_members,cards,lists,board_members,boards,users CASCADE")
            .execute(&pool).await.ok();
        crate::db::run_migrations(&pool).await.unwrap();

        let uid = Uuid::new_v4();
        let now = chrono::Utc::now().to_rfc3339();
        sqlx::query("INSERT INTO users (id,email,password_hash,name,created_at,updated_at) VALUES($1,$2,$3,$4,$5,$6)")
            .bind(uid).bind("t@t.com").bind("hash").bind("T").bind(&now).bind(&now)
            .execute(&pool).await.unwrap();
        (pool, uid)
    }

    #[tokio::test]
    async fn test_create_card_tool() {
        let (pool, uid) = setup_pool().await;
        let row = sqlx::query("INSERT INTO boards(name,created_by)VALUES($1,$2)RETURNING id")
            .bind("Board").bind(uid).fetch_one(&pool).await.unwrap();
        let bid: Uuid = row.get("id");
        sqlx::query("INSERT INTO lists(board_id,name,position,list_type)VALUES($1,$2,$3,$4)")
            .bind(bid).bind("To Do").bind(0.0).bind("active")
            .execute(&pool).await.unwrap();
        let list: (String,) = sqlx::query_as("SELECT id::text FROM lists WHERE board_id=$1 ORDER BY position LIMIT 1")
            .bind(bid).fetch_one(&pool).await.unwrap();

        let result = execute_tool("create_card", &json!({"list_id": list.0, "name": "AI Card"}), &uid, &pool).await;
        assert!(result.success, "create_card failed: {:?}", result.data);
        assert_eq!(result.data["name"], "AI Card");
    }

    #[tokio::test]
    async fn test_create_card_missing_name() {
        let (pool, uid) = setup_pool().await;
        let result = execute_tool("create_card", &json!({"list_id": Uuid::new_v4().to_string()}), &uid, &pool).await;
        assert!(!result.success);
    }

    #[tokio::test]
    async fn test_unknown_tool() {
        let (pool, uid) = setup_pool().await;
        let result = execute_tool("nonexistent", &json!({}), &uid, &pool).await;
        assert!(!result.success);
    }
}
