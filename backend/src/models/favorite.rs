use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Favorite {
    pub id: String,
    pub user_id: String,
    pub board_id: Option<String>,
    pub card_id: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateFavoriteRequest {
    pub board_id: Option<String>,
    pub card_id: Option<String>,
}
