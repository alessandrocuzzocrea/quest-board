use uuid::Uuid;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Favorite {
    pub id: Uuid,
    pub user_id: Uuid,
    pub board_id: Option<Uuid>,
    pub card_id: Option<Uuid>,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateFavoriteRequest {
    pub board_id: Option<Uuid>,
    pub card_id: Option<Uuid>,
}
