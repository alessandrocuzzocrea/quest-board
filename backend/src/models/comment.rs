use uuid::Uuid;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Comment {
    pub id: Uuid,
    pub card_id: Uuid,
    pub user_id: Uuid,
    pub text: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateCommentRequest {
    pub card_id: Uuid,
    pub text: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCommentRequest {
    pub text: String,
}

#[derive(Debug, Serialize)]
pub struct CommentWithUser {
    pub id: Uuid,
    pub card_id: Uuid,
    pub user_id: Uuid,
    pub user: Option<super::user::UserResponse>,
    pub text: String,
    pub created_at: String,
    pub updated_at: String,
}
