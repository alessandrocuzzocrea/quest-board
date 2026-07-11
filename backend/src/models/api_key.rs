use uuid::Uuid;

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use ts_rs::TS;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, TS)]
#[ts(export)]
pub struct ApiKey {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    #[serde(skip_serializing)]
    #[ts(skip)]
    pub token_hash: String,
    pub prefix: String,
    #[ts(type = "string | null")]
    pub expires_at: Option<OffsetDateTime>,
    #[ts(type = "string | null")]
    pub last_used_at: Option<OffsetDateTime>,
    #[ts(type = "string")]
    pub created_at: OffsetDateTime,
    pub is_active: bool,
}

#[derive(Debug, Deserialize, TS)]
#[ts(export)]
pub struct CreateApiKeyRequest {
    pub name: String,
    #[ts(type = "string | null")]
    pub expires_at: Option<OffsetDateTime>,
}

#[derive(Debug, Serialize, TS)]
#[ts(export)]
pub struct ApiKeyResponse {
    pub id: Uuid,
    pub name: String,
    pub prefix: String,
    #[ts(type = "string | null")]
    pub expires_at: Option<OffsetDateTime>,
    #[ts(type = "string | null")]
    pub last_used_at: Option<OffsetDateTime>,
    #[ts(type = "string")]
    pub created_at: OffsetDateTime,
    pub is_active: bool,
}

impl From<ApiKey> for ApiKeyResponse {
    fn from(k: ApiKey) -> Self {
        ApiKeyResponse {
            id: k.id,
            name: k.name,
            prefix: k.prefix,
            expires_at: k.expires_at,
            last_used_at: k.last_used_at,
            created_at: k.created_at,
            is_active: k.is_active,
        }
    }
}
