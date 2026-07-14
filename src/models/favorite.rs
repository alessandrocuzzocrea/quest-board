use uuid::Uuid;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, TS)]
#[ts(export)]
pub struct Favorite {
    pub id: Uuid,
    pub user_id: Uuid,
    pub board_id: Option<Uuid>,
    pub card_id: Option<Uuid>,
    pub created_at: String,
}

#[derive(Debug, Deserialize, TS)]
#[ts(export)]
pub struct CreateFavoriteRequest {
    pub board_id: Option<Uuid>,
    pub card_id: Option<Uuid>,
}
