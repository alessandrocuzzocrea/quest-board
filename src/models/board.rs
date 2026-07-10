use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Board {
    pub id: String,
    pub name: String,
    pub position: f64,
    pub created_by: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateBoardRequest {
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateBoardRequest {
    pub name: Option<String>,
    pub position: Option<f64>,
}

#[derive(Debug, Serialize)]
pub struct BoardWithLists {
    pub id: String,
    pub name: String,
    pub position: f64,
    pub created_by: String,
    pub lists: Vec<super::list::List>,
    pub members: Vec<super::user::UserResponse>,
    pub created_at: String,
    pub updated_at: String,
}
