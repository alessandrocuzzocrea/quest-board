use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Attachment {
    pub id: String,
    pub card_id: String,
    pub user_id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub attachment_type: String,
    pub file_path: Option<String>,
    pub link_url: Option<String>,
    pub size: Option<i64>,
    pub mime_type: Option<String>,
    pub created_at: String,
}
