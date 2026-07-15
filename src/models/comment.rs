use uuid::Uuid;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, TS, utoipa::ToSchema)]
#[ts(export)]
pub struct Comment {
    pub id: Uuid,
    pub card_id: Uuid,
    pub user_id: Uuid,
    pub text: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize, TS, utoipa::ToSchema)]
#[ts(export)]
pub struct CreateCommentRequest {
    pub card_id: Uuid,
    pub text: String,
}

#[derive(Debug, Deserialize, TS, utoipa::ToSchema)]
#[ts(export)]
pub struct UpdateCommentRequest {
    pub text: String,
}

#[derive(Debug, Serialize, TS, utoipa::ToSchema)]
#[ts(export)]
pub struct CommentWithUser {
    pub id: Uuid,
    pub card_id: Uuid,
    pub user_id: Uuid,
    pub user: Option<super::user::UserResponse>,
    pub text: String,
    pub created_at: String,
    pub updated_at: String,
}
