use crate::error::AppError;
use crate::models::checklist::{TaskList, TaskListWithTasks, Task};

pub async fn create_task_list(
    pool: &sqlx::PgPool,
    id: &str,
    card_id: &str,
    name: &str,
    position: f64,
) -> Result<TaskList, AppError> {
    sqlx::query(
        "INSERT INTO task_lists (id, card_id, name, position) VALUES ($1, $2, $3, $4)",
    )
    .bind(id)
    .bind(card_id)
    .bind(name)
    .bind(position)
    .execute(pool)
    .await?;
    task_list_by_id(pool, id).await.transpose().unwrap()
}

pub async fn task_list_by_id(pool: &sqlx::PgPool, id: &str) -> Result<Option<TaskList>, AppError> {
    Ok(sqlx::query_as("SELECT * FROM task_lists WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await?)
}

pub async fn update_task_list_name(pool: &sqlx::PgPool, id: &str, name: &str) -> Result<(), AppError> {
    sqlx::query("UPDATE task_lists SET name = $1, updated_at = NOW() WHERE id = $2")
        .bind(name).bind(id).execute(pool).await?;
    Ok(())
}

pub async fn delete_task_list(pool: &sqlx::PgPool, id: &str) -> Result<(), AppError> {
    sqlx::query("DELETE FROM task_lists WHERE id = $1")
        .bind(id).execute(pool).await?;
    Ok(())
}

pub async fn create_task(
    pool: &sqlx::PgPool,
    id: &str,
    task_list_id: &str,
    name: &str,
    position: f64,
) -> Result<Task, AppError> {
    sqlx::query(
        "INSERT INTO tasks (id, task_list_id, name, position) VALUES ($1, $2, $3, $4)",
    )
    .bind(id)
    .bind(task_list_id)
    .bind(name)
    .bind(position)
    .execute(pool)
    .await?;
    task_by_id(pool, id).await.transpose().unwrap()
}

pub async fn task_by_id(pool: &sqlx::PgPool, id: &str) -> Result<Option<Task>, AppError> {
    Ok(sqlx::query_as("SELECT * FROM tasks WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await?)
}

pub async fn update_task(
    pool: &sqlx::PgPool,
    id: &str,
    name: Option<&str>,
    is_completed: Option<bool>,
    position: Option<f64>,
    assignee_id: Option<&str>,
) -> Result<Task, AppError> {
    if let Some(name) = name {
        sqlx::query("UPDATE tasks SET name = $1, updated_at = NOW() WHERE id = $2")
            .bind(name).bind(id).execute(pool).await?;
    }
    if let Some(completed) = is_completed {
        sqlx::query("UPDATE tasks SET is_completed = $1, updated_at = NOW() WHERE id = $2")
            .bind(completed).bind(id).execute(pool).await?;
    }
    if let Some(pos) = position {
        sqlx::query("UPDATE tasks SET position = $1, updated_at = NOW() WHERE id = $2")
            .bind(pos).bind(id).execute(pool).await?;
    }
    if let Some(ass_id) = assignee_id {
        sqlx::query("UPDATE tasks SET assignee_id = $1, updated_at = NOW() WHERE id = $2")
            .bind(ass_id).bind(id).execute(pool).await?;
    }
    task_by_id(pool, id).await.transpose().unwrap()
}

pub async fn delete_task(pool: &sqlx::PgPool, id: &str) -> Result<(), AppError> {
    sqlx::query("DELETE FROM tasks WHERE id = $1")
        .bind(id).execute(pool).await?;
    Ok(())
}

pub async fn list_by_card(pool: &sqlx::PgPool, card_id: &str) -> Result<Vec<TaskListWithTasks>, AppError> {
    let task_lists: Vec<TaskList> = sqlx::query_as(
        "SELECT * FROM task_lists WHERE card_id = $1 ORDER BY position",
    )
    .bind(card_id)
    .fetch_all(pool)
    .await?;

    let mut result: Vec<TaskListWithTasks> = Vec::new();
    for tl in task_lists {
        let tasks: Vec<Task> = sqlx::query_as(
            "SELECT * FROM tasks WHERE task_list_id = $1 ORDER BY position",
        )
        .bind(&tl.id)
        .fetch_all(pool)
        .await?;
        result.push(TaskListWithTasks {
            id: tl.id,
            card_id: tl.card_id,
            name: tl.name,
            position: tl.position,
            hide_completed: tl.hide_completed,
            tasks,
            created_at: tl.created_at,
            updated_at: tl.updated_at,
        });
    }
    Ok(result)
}
