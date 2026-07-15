use uuid::Uuid;

use crate::error::AppError;
use crate::models::api_key::{ApiKeyResponse, CreateApiKeyRequest};
use crate::repository;

/// Business logic for API keys, shared by HTML and JSON handlers.
pub struct ApiKeyService {
    db: sqlx::PgPool,
}

impl ApiKeyService {
    pub fn new(db: sqlx::PgPool) -> Self {
        Self { db }
    }

    pub async fn list_by_user(&self, user_id: Uuid) -> Result<Vec<ApiKeyResponse>, AppError> {
        let keys = repository::api_key_repo::list_by_user(&self.db, user_id).await?;
        Ok(keys.into_iter().map(ApiKeyResponse::from).collect())
    }

    pub async fn create(&self, user_id: Uuid, req: &CreateApiKeyRequest) -> Result<(ApiKeyResponse, String), AppError> {
        let (full_token, prefix, token_hash) = crate::auth::generate_api_key();

        let key = repository::api_key_repo::create(
            &self.db, user_id, &req.name, &token_hash, &prefix, req.expires_at,
        )
        .await?;

        Ok((ApiKeyResponse::from(key), full_token))
    }

    pub async fn delete(&self, key_id: Uuid, user_id: Uuid) -> Result<(), AppError> {
        repository::api_key_repo::delete(&self.db, key_id, user_id).await
    }
}
