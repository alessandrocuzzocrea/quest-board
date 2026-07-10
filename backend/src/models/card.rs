use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Card {
    pub id: String,
    pub board_id: String,
    pub list_id: String,
    pub position: f64,
    pub name: String,
    pub description: Option<String>,
    pub due_date: Option<String>,
    pub is_due_completed: bool,
    pub is_closed: bool,
    pub created_by: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateCardRequest {
    pub list_id: String,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCardRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub position: Option<f64>,
    pub due_date: Option<String>,
    pub is_due_completed: Option<bool>,
    pub is_closed: Option<bool>,
    pub list_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CardWithMembers {
    pub id: String,
    pub board_id: String,
    pub list_id: String,
    pub position: f64,
    pub name: String,
    pub description: Option<String>,
    pub due_date: Option<String>,
    pub is_due_completed: bool,
    pub is_closed: bool,
    pub created_by: String,
    pub members: Vec<super::user::UserResponse>,
    pub labels: Vec<super::label::Label>,
    pub comments_count: i64,
    pub checklists: Vec<super::checklist::TaskListWithTasks>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct MoveCardRequest {
    pub list_id: String,
    pub position: f64,
}
