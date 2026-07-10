use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct List {
    pub id: String,
    pub board_id: String,
    pub name: Option<String>,
    pub position: f64,
    #[serde(rename = "type")]
    pub list_type: String,
    pub color: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateListRequest {
    pub board_id: String,
    pub name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateListRequest {
    pub name: Option<String>,
    pub position: Option<f64>,
    pub color: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ListWithCards {
    pub id: String,
    pub board_id: String,
    pub name: Option<String>,
    pub position: f64,
    #[serde(rename = "type")]
    pub list_type: String,
    pub color: Option<String>,
    pub cards: Vec<super::card::CardWithMembers>,
    pub created_at: String,
    pub updated_at: String,
}
