use uuid::Uuid;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, TS, utoipa::ToSchema)]
#[ts(export)]
pub struct List {
    pub id: Uuid,
    pub board_id: Uuid,
    pub name: Option<String>,
    pub position: f64,
    #[serde(rename = "type")]
    pub list_type: String,
    pub color: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize, TS, utoipa::ToSchema)]
#[ts(export)]
pub struct CreateListRequest {
    pub board_id: Uuid,
    pub name: Option<String>,
}

#[derive(Debug, Deserialize, TS, utoipa::ToSchema)]
#[ts(export)]
pub struct UpdateListRequest {
    pub name: Option<String>,
    pub position: Option<f64>,
    pub color: Option<String>,
}

#[derive(Debug, Serialize, TS, utoipa::ToSchema)]
#[ts(export)]
pub struct ListWithCards {
    pub id: Uuid,
    pub board_id: Uuid,
    pub name: Option<String>,
    pub position: f64,
    #[serde(rename = "type")]
    pub list_type: String,
    pub color: Option<String>,
    pub cards: Vec<super::card::CardWithMembers>,
    pub created_at: String,
    pub updated_at: String,
}
