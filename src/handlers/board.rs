use axum::extract::{Path, State};
use axum::routing::get;
use axum::{Json, Router};
use std::sync::Arc;

use crate::error::AppError;
use crate::models::board::*;
use crate::models::list::{List, ListWithCards};
use crate::models::card::{Card, CardWithMembers};
use crate::AppState;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(list_boards).post(create_board))
        .route("/{id}", get(get_board).put(update_board).delete(delete_board))
}

async fn require_user(session: &tower_sessions::Session) -> Result<String, AppError> {
    session
        .get::<String>("user_id")
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .ok_or(AppError::Unauthorized("not logged in".into()))
}

async fn list_boards(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
) -> Result<Json<serde_json::Value>, AppError> {
    let user_id = require_user(&session).await?;

    let boards: Vec<Board> = sqlx::query_as(
        "SELECT b.* FROM boards b
         LEFT JOIN board_members bm ON b.id = bm.board_id AND bm.user_id = ?
         WHERE b.created_by = ? OR bm.user_id = ?
         ORDER BY b.position, b.created_at",
    )
    .bind(&user_id)
    .bind(&user_id)
    .bind(&user_id)
    .fetch_all(&state.db)
    .await?;

    Ok(Json(serde_json::json!(boards)))
}

async fn create_board(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    Json(req): Json<CreateBoardRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let user_id = require_user(&session).await?;
    let id = uuid::Uuid::new_v4().to_string();

    sqlx::query("INSERT INTO boards (id, name, created_by) VALUES (?, ?, ?)")
        .bind(&id)
        .bind(&req.name)
        .bind(&user_id)
        .execute(&state.db)
        .await?;

    // Add creator as admin member
    sqlx::query("INSERT INTO board_members (id, board_id, user_id, role) VALUES (?, ?, ?, 'admin')")
        .bind(uuid::Uuid::new_v4().to_string())
        .bind(&id)
        .bind(&user_id)
        .execute(&state.db)
        .await?;

    // Create default lists: To Do, In Progress, Done
    let now = chrono::Utc::now().to_rfc3339();
    for (i, name) in ["To Do", "In Progress", "Done"].iter().enumerate() {
        sqlx::query(
            "INSERT INTO lists (id, board_id, name, position, list_type) VALUES (?, ?, ?, ?, 'active')",
        )
        .bind(uuid::Uuid::new_v4().to_string())
        .bind(&id)
        .bind(name)
        .bind(i as f64 * 65536.0)
        .execute(&state.db)
        .await?;
    }

    let board: Board = sqlx::query_as("SELECT * FROM boards WHERE id = ?")
        .bind(&id)
        .fetch_one(&state.db)
        .await?;

    Ok(Json(serde_json::json!(board)))
}

async fn get_board(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    Path(board_id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let _user_id = require_user(&session).await?;

    let board: Board = sqlx::query_as("SELECT * FROM boards WHERE id = ?")
        .bind(&board_id)
        .fetch_optional(&state.db)
        .await?
        .ok_or(AppError::NotFound("board not found".into()))?;

    let lists: Vec<List> = sqlx::query_as(
        "SELECT * FROM lists WHERE board_id = ? ORDER BY position, created_at",
    )
    .bind(&board_id)
    .fetch_all(&state.db)
    .await?;

    let members: Vec<crate::models::user::User> = sqlx::query_as(
        "SELECT u.* FROM users u
         JOIN board_members bm ON u.id = bm.user_id
         WHERE bm.board_id = ?",
    )
    .bind(&board_id)
    .fetch_all(&state.db)
    .await?;

    // Build lists with cards
    let mut list_with_cards: Vec<ListWithCards> = Vec::new();
    for list in lists {
        let cards: Vec<Card> = sqlx::query_as(
            "SELECT * FROM cards WHERE list_id = ? ORDER BY position, created_at",
        )
        .bind(&list.id)
        .fetch_all(&state.db)
        .await?;

        let mut card_with_members: Vec<CardWithMembers> = Vec::new();
        for card in cards {
            let members: Vec<crate::models::user::User> = sqlx::query_as(
                "SELECT u.* FROM users u
                 JOIN card_members cm ON u.id = cm.user_id
                 WHERE cm.card_id = ?",
            )
            .bind(&card.id)
            .fetch_all(&state.db)
            .await?;

            let labels: Vec<crate::models::label::Label> = sqlx::query_as(
                "SELECT l.* FROM labels l
                 JOIN card_labels cl ON l.id = cl.label_id
                 WHERE cl.card_id = ?",
            )
            .bind(&card.id)
            .fetch_all(&state.db)
            .await?;

            let comments_count: (i64,) = sqlx::query_as(
                "SELECT COUNT(*) FROM comments WHERE card_id = ?",
            )
            .bind(&card.id)
            .fetch_one(&state.db)
            .await?;

            let checklists: Vec<crate::models::checklist::TaskList> = sqlx::query_as(
                "SELECT * FROM task_lists WHERE card_id = ? ORDER BY position",
            )
            .bind(&card.id)
            .fetch_all(&state.db)
            .await?;

            let mut task_list_with_tasks: Vec<crate::models::checklist::TaskListWithTasks> = Vec::new();
            for tl in checklists {
                let tasks: Vec<crate::models::checklist::Task> = sqlx::query_as(
                    "SELECT * FROM tasks WHERE task_list_id = ? ORDER BY position",
                )
                .bind(&tl.id)
                .fetch_all(&state.db)
                .await?;
                task_list_with_tasks.push(crate::models::checklist::TaskListWithTasks {
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

            card_with_members.push(CardWithMembers {
                id: card.id,
                board_id: card.board_id,
                list_id: card.list_id,
                position: card.position,
                name: card.name,
                description: card.description,
                due_date: card.due_date,
                is_due_completed: card.is_due_completed,
                is_closed: card.is_closed,
                created_by: card.created_by,
                members: members.into_iter().map(Into::into).collect(),
                labels,
                comments_count: comments_count.0,
                checklists: task_list_with_tasks,
                created_at: card.created_at,
                updated_at: card.updated_at,
            });
        }

        list_with_cards.push(ListWithCards {
            id: list.id,
            board_id: list.board_id,
            name: list.name,
            position: list.position,
            list_type: list.list_type,
            color: list.color,
            cards: card_with_members,
            created_at: list.created_at,
            updated_at: list.updated_at,
        });
    }

    Ok(Json(serde_json::json!({
        "board": {
            "id": board.id,
            "name": board.name,
            "position": board.position,
            "created_by": board.created_by,
            "created_at": board.created_at,
            "updated_at": board.updated_at,
        },
        "lists": list_with_cards,
        "members": members.into_iter().map(crate::models::user::UserResponse::from).collect::<Vec<_>>(),
    })))
}

async fn update_board(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    Path(board_id): Path<String>,
    Json(req): Json<UpdateBoardRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let _user_id = require_user(&session).await?;

    if let Some(name) = &req.name {
        sqlx::query("UPDATE boards SET name = ?, updated_at = datetime('now') WHERE id = ?")
            .bind(name)
            .bind(&board_id)
            .execute(&state.db)
            .await?;
    }
    if let Some(position) = req.position {
        sqlx::query("UPDATE boards SET position = ?, updated_at = datetime('now') WHERE id = ?")
            .bind(position)
            .bind(&board_id)
            .execute(&state.db)
            .await?;
    }

    let board: Board = sqlx::query_as("SELECT * FROM boards WHERE id = ?")
        .bind(&board_id)
        .fetch_optional(&state.db)
        .await?
        .ok_or(AppError::NotFound("board not found".into()))?;

    Ok(Json(serde_json::json!(board)))
}

async fn delete_board(
    State(state): State<Arc<AppState>>,
    session: tower_sessions::Session,
    Path(board_id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let _user_id = require_user(&session).await?;
    sqlx::query("DELETE FROM boards WHERE id = ?")
        .bind(&board_id)
        .execute(&state.db)
        .await?;
    Ok(Json(serde_json::json!({"ok": true})))
}
