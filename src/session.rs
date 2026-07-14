use async_trait::async_trait;
use std::collections::HashMap;
use time::OffsetDateTime;
use tower_sessions_core::session::{Id, Record};
use tower_sessions_core::session_store;
use tower_sessions_core::SessionStore;

#[derive(Clone, Debug)]
pub struct PgSessionStore {
    pool: sqlx::PgPool,
}

impl PgSessionStore {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SessionStore for PgSessionStore {
    async fn create(&self, record: &mut Record) -> session_store::Result<()> {
        sqlx::query(
            "INSERT INTO sessions (id, data, expiry_date) VALUES ($1, $2, $3)",
        )
        .bind(record.id.0.to_string())
        .bind(serde_json::to_value(&record.data).unwrap_or_default())
        .bind(record.expiry_date)
        .execute(&self.pool)
        .await
        .map_err(|e| session_store::Error::Backend(e.to_string()))?;

        Ok(())
    }

    async fn save(&self, record: &Record) -> session_store::Result<()> {
        sqlx::query(
            "UPDATE sessions SET data = $1, expiry_date = $2 WHERE id = $3",
        )
        .bind(serde_json::to_value(&record.data).unwrap_or_default())
        .bind(record.expiry_date)
        .bind(record.id.0.to_string())
        .execute(&self.pool)
        .await
        .map_err(|e| session_store::Error::Backend(e.to_string()))?;

        Ok(())
    }

    async fn load(&self, session_id: &Id) -> session_store::Result<Option<Record>> {
        let row: Option<(serde_json::Value, OffsetDateTime)> = sqlx::query_as(
            "SELECT data, expiry_date FROM sessions WHERE id = $1",
        )
        .bind(session_id.0.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| session_store::Error::Backend(e.to_string()))?;

        match row {
            Some((data, expiry_date)) => {
                let data_map: HashMap<String, serde_json::Value> =
                    serde_json::from_value(data).unwrap_or_default();
                Ok(Some(Record {
                    id: *session_id,
                    data: data_map,
                    expiry_date,
                }))
            }
            None => Ok(None),
        }
    }

    async fn delete(&self, session_id: &Id) -> session_store::Result<()> {
        sqlx::query("DELETE FROM sessions WHERE id = $1")
            .bind(session_id.0.to_string())
            .execute(&self.pool)
            .await
            .map_err(|e| session_store::Error::Backend(e.to_string()))?;

        Ok(())
    }
}
