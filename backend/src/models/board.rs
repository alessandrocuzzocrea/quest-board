use uuid::Uuid;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, TS)]
#[ts(export)]
pub struct Board {
    pub id: Uuid,
    pub name: String,
    pub position: f64,
    pub slug: String,
    pub created_by: Uuid,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize, TS)]
#[ts(export)]
pub struct CreateBoardRequest {
    pub name: String,
}

#[derive(Debug, Deserialize, TS)]
#[ts(export)]
pub struct UpdateBoardRequest {
    pub name: Option<String>,
    pub position: Option<f64>,
}

