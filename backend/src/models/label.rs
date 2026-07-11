use uuid::Uuid;

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, ToSchema)]
pub struct Label {
    pub id: Uuid,
    pub board_id: Uuid,
    pub name: String,
    pub color: String,
    pub position: f64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateLabelRequest {
    pub board_id: Uuid,
    pub name: String,
    pub color: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateLabelRequest {
    pub name: Option<String>,
    pub color: Option<String>,
    pub position: Option<f64>,
}
