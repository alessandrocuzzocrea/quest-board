use uuid::Uuid;

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, ToSchema)]
pub struct Attachment {
    pub id: Uuid,
    pub card_id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    #[serde(rename = "type")]
    pub attachment_type: String,
    pub file_path: Option<String>,
    pub link_url: Option<String>,
    pub size: Option<i64>,
    pub mime_type: Option<String>,
    pub created_at: String,
}
