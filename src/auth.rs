use base64::Engine;
use rand::rngs::SysRng;
use rand::TryRng;
use sha2::Digest;
use crate::error::AppError;

/// Generates a new API key and returns (full_token, prefix, token_hash).
///
/// Token format: `qb_` + 43 random base64url characters (32 random bytes,
/// URL_SAFE_NO_PAD encoded → 43 chars).
/// Prefix: first 8 characters of the encoded portion (after `qb_`).
/// Token hash: SHA-256 of the full token string, hex-encoded.
pub fn generate_api_key() -> (String, String, String) {
    let mut bytes = [0u8; 32];
    SysRng.try_fill_bytes(&mut bytes).expect("SysRng failed");

    let encoded =
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&bytes);
    let full = format!("qb_{}", encoded);
    let prefix = encoded[..8].to_string();
    let hash = hex::encode(sha2::Sha256::digest(&full));

    (full, prefix, hash)
}

/// Resolves the authenticated user from the request.
///
/// Checks the session first for a stored `user_id`, then falls back to
/// a Bearer token in the `Authorization` header.  The bearer token is
/// hashed with SHA-256 and looked up in the `api_keys` table.
pub async fn resolve_user(
    session: &tower_sessions::Session,
    headers: &axum::http::HeaderMap,
    pool: &sqlx::PgPool,
) -> Result<uuid::Uuid, AppError> {
    // Check session first
    if let Ok(Some(uid)) = session.get::<String>("user_id").await {
        if let Ok(uid) = uuid::Uuid::parse_str(&uid) {
            return Ok(uid);
        }
    }

    // Check bearer token
    if let Some(auth) = headers.get("authorization") {
        if let Ok(auth_str) = auth.to_str() {
            if let Some(token) = auth_str.strip_prefix("Bearer ") {
                let hash = hex::encode(sha2::Sha256::digest(token));
                // Look up token in api_keys table
                let row: Option<(uuid::Uuid,)> = sqlx::query_as(
                    "SELECT user_id FROM api_keys WHERE token_hash = $1 AND is_active = true AND (expires_at IS NULL OR expires_at > NOW())",
                )
                .bind(&hash)
                .fetch_optional(pool)
                .await
                .map_err(|e| AppError::Internal(e.to_string()))?;

                if let Some((uid,)) = row {
                    // Update last_used_at
                    let _ = sqlx::query(
                        "UPDATE api_keys SET last_used_at = NOW() WHERE token_hash = $1",
                    )
                    .bind(&hash)
                    .execute(pool)
                    .await;
                    return Ok(uid);
                }
            }
        }
    }

    Err(AppError::Unauthorized("not logged in".into()))
}
