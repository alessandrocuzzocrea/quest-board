use uuid::Uuid;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, TS, utoipa::ToSchema)]
#[ts(export)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    #[serde(skip_serializing)]
    #[ts(skip)]
    pub password_hash: String,
    pub role: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize, TS, utoipa::ToSchema)]
#[ts(export)]
pub struct RegisterRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize, TS, utoipa::ToSchema)]
#[ts(export)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize, TS, utoipa::ToSchema)]
#[ts(export)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    pub new_password: String,
}

#[derive(Debug, Serialize, TS, utoipa::ToSchema)]
#[ts(export)]
pub struct UserResponse {
    pub id: Uuid,
    pub username: String,
    pub role: String,
}

impl From<User> for UserResponse {
    fn from(u: User) -> Self {
        UserResponse {
            id: u.id,
            username: u.username,
            role: u.role,
        }
    }
}
