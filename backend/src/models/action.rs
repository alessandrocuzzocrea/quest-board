use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Action {
    pub id: String,
    pub card_id: String,
    pub board_id: Option<String>,
    pub user_id: Option<String>,
    #[serde(rename = "type")]
    pub action_type: String,
    pub data: String,
    pub created_at: String,
}
