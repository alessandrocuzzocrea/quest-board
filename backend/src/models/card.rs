use uuid::Uuid;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, TS)]
#[ts(export)]
pub struct Card {
    pub id: Uuid,
    pub board_id: Uuid,
    pub list_id: Uuid,
    pub position: f64,
    pub name: String,
    pub description: Option<String>,
    pub due_date: Option<String>,
    pub is_due_completed: bool,
    pub is_closed: bool,
    pub created_by: Uuid,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize, TS)]
#[ts(export)]
pub struct CreateCardRequest {
    pub list_id: Uuid,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, TS)]
#[ts(export)]
pub struct UpdateCardRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub position: Option<f64>,
    pub due_date: Option<String>,
    pub is_due_completed: Option<bool>,
    pub is_closed: Option<bool>,
    pub list_id: Option<Uuid>,
}

#[derive(Debug, Serialize, TS)]
#[ts(export)]
pub struct CardWithMembers {
    pub id: Uuid,
    pub board_id: Uuid,
    pub list_id: Uuid,
    pub position: f64,
    pub name: String,
    pub description: Option<String>,
    pub due_date: Option<String>,
    pub is_due_completed: bool,
    pub is_closed: bool,
    pub created_by: Uuid,
    pub members: Vec<super::user::UserResponse>,
    pub labels: Vec<super::label::Label>,
    pub comments_count: i64,
    pub checklists: Vec<super::checklist::TaskListWithTasks>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize, TS)]
#[ts(export)]
pub struct MoveCardRequest {
    pub list_id: Uuid,
    pub position: f64,
}
