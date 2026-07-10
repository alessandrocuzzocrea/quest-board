use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct TaskList {
    pub id: String,
    pub card_id: String,
    pub name: String,
    pub position: f64,
    pub hide_completed: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Task {
    pub id: String,
    pub task_list_id: String,
    pub name: String,
    pub position: f64,
    pub is_completed: bool,
    pub assignee_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize)]
pub struct TaskListWithTasks {
    pub id: String,
    pub card_id: String,
    pub name: String,
    pub position: f64,
    pub hide_completed: bool,
    pub tasks: Vec<Task>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateTaskListRequest {
    pub card_id: String,
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateTaskRequest {
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTaskRequest {
    pub name: Option<String>,
    pub is_completed: Option<bool>,
    pub position: Option<f64>,
    pub assignee_id: Option<String>,
}
