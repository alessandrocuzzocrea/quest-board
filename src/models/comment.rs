use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Comment {
    pub id: String,
    pub card_id: String,
    pub user_id: String,
    pub text: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateCommentRequest {
    pub card_id: String,
    pub text: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCommentRequest {
    pub text: String,
}

#[derive(Debug, Serialize)]
pub struct CommentWithUser {
    pub id: String,
    pub card_id: String,
    pub user_id: String,
    pub user: Option<super::user::UserResponse>,
    pub text: String,
    pub created_at: String,
    pub updated_at: String,
}
