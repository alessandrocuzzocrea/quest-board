use uuid::Uuid;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, TS)]
#[ts(export)]
pub struct Notification {
    pub id: Uuid,
    pub user_id: Uuid,
    #[serde(rename = "type")]
    pub notif_type: String,
    pub data: String,
    pub is_read: bool,
    pub card_id: Option<Uuid>,
    pub action_id: Option<Uuid>,
    pub created_at: String,
}
