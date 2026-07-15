use uuid::Uuid;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, TS, utoipa::ToSchema)]
#[ts(export)]
pub struct Action {
    pub id: Uuid,
    pub card_id: Uuid,
    pub board_id: Option<Uuid>,
    pub user_id: Option<Uuid>,
    #[serde(rename = "type")]
    pub action_type: String,
    pub data: String,
    pub created_at: String,
}
