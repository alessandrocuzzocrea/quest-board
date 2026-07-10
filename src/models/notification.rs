use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Notification {
    pub id: String,
    pub user_id: String,
    #[serde(rename = "type")]
    pub notif_type: String,
    pub data: String,
    pub is_read: bool,
    pub card_id: Option<String>,
    pub action_id: Option<String>,
    pub created_at: String,
}
