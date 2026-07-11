use uuid::Uuid;

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, ToSchema)]
pub struct Favorite {
    pub id: Uuid,
    pub user_id: Uuid,
    pub board_id: Option<Uuid>,
    pub card_id: Option<Uuid>,
    pub created_at: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateFavoriteRequest {
    pub board_id: Option<Uuid>,
    pub card_id: Option<Uuid>,
}
