use crate::error::AppError;
use crate::models::board::Board;
use crate::models::list::{List, ListWithCards};
use crate::models::card::{Card, CardWithMembers};
use crate::models::user::UserResponse;
use uuid::Uuid;

pub async fn list_accessible(pool: &sqlx::PgPool, user_id: &Uuid) -> Result<Vec<Board>, AppError> {
    Ok(sqlx::query_as(
        "SELECT b.* FROM boards b
         LEFT JOIN board_members bm ON b.id = bm.board_id AND bm.user_id = $1
         WHERE b.created_by = $2 OR bm.user_id = $3
         ORDER BY b.position, b.created_at",
    )
    .bind(user_id)
    .bind(user_id)
    .bind(user_id)
    .fetch_all(pool)
    .await?)
}

pub async fn get_by_id(pool: &sqlx::PgPool, board_id: &Uuid) -> Result<Option<Board>, AppError> {
    Ok(sqlx::query_as("SELECT * FROM boards WHERE id = $1")
        .bind(board_id)
        .fetch_optional(pool)
        .await?)
}

pub async fn get_by_slug(pool: &sqlx::PgPool, slug: &str) -> Result<Option<Board>, AppError> {
    Ok(sqlx::query_as("SELECT * FROM boards WHERE slug = $1")
        .bind(slug)
        .fetch_optional(pool)
        .await?)
}

pub async fn create(pool: &sqlx::PgPool, name: &str, created_by: &Uuid) -> Result<Board, AppError> {
    let slug = crate::slug::generate_slug();
    let id: Uuid = sqlx::query_scalar(
        "INSERT INTO boards (name, created_by, slug) VALUES ($1, $2, $3) RETURNING id",
    )
    .bind(name)
    .bind(created_by)
    .bind(&slug)
    .fetch_one(pool)
    .await?;
    get_by_id(pool, &id).await.transpose().unwrap()
}

pub async fn add_member(pool: &sqlx::PgPool, board_id: &Uuid, user_id: &Uuid, role: &str) -> Result<(), AppError> {
    sqlx::query("INSERT INTO board_members (board_id, user_id, role) VALUES ($1, $2, $3)")
        .bind(board_id)
        .bind(user_id)
        .bind(role)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn update_name(pool: &sqlx::PgPool, board_id: &Uuid, name: &str) -> Result<(), AppError> {
    sqlx::query("UPDATE boards SET name = $1, updated_at = NOW() WHERE id = $2")
        .bind(name)
        .bind(board_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn update_position(pool: &sqlx::PgPool, board_id: &Uuid, position: f64) -> Result<(), AppError> {
    sqlx::query("UPDATE boards SET position = $1, updated_at = NOW() WHERE id = $2")
        .bind(position)
        .bind(board_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn delete(pool: &sqlx::PgPool, board_id: &Uuid) -> Result<(), AppError> {
    sqlx::query("DELETE FROM boards WHERE id = $1")
        .bind(board_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn list_members(pool: &sqlx::PgPool, board_id: &Uuid) -> Result<Vec<UserResponse>, AppError> {
    let users: Vec<crate::models::user::User> = sqlx::query_as(
        "SELECT u.* FROM users u
         JOIN board_members bm ON u.id = bm.user_id
         WHERE bm.board_id = $1",
    )
    .bind(board_id)
    .fetch_all(pool)
    .await?;
    Ok(users.into_iter().map(Into::into).collect())
}

pub async fn get_full_board(
    pool: &sqlx::PgPool,
    board_id: &Uuid,
) -> Result<(Board, Vec<ListWithCards>, Vec<UserResponse>), AppError> {
    let board = get_by_id(pool, board_id)
        .await?
        .ok_or(AppError::NotFound("board not found".into()))?;

    let lists: Vec<List> = sqlx::query_as(
        "SELECT * FROM lists WHERE board_id = $1 ORDER BY position, created_at",
    )
    .bind(board_id)
    .fetch_all(pool)
    .await?;

    let members = list_members(pool, board_id).await?;

    let mut list_with_cards: Vec<ListWithCards> = Vec::new();
    for list in lists {
        let cards: Vec<Card> = sqlx::query_as(
            "SELECT * FROM cards WHERE list_id = $1 ORDER BY position, created_at",
        )
        .bind(&list.id)
        .fetch_all(pool)
        .await?;

        let mut card_with_members: Vec<CardWithMembers> = Vec::new();
        for card in cards {
            let card_members: Vec<UserResponse> = {
                let users: Vec<crate::models::user::User> = sqlx::query_as(
                    "SELECT u.* FROM users u
                     JOIN card_members cm ON u.id = cm.user_id
                     WHERE cm.card_id = $1",
                )
                .bind(&card.id)
                .fetch_all(pool)
                .await?;
                users.into_iter().map(Into::into).collect()
            };

            let labels: Vec<crate::models::label::Label> = sqlx::query_as(
                "SELECT l.* FROM labels l
                 JOIN card_labels cl ON l.id = cl.label_id
                 WHERE cl.card_id = $1",
            )
            .bind(&card.id)
            .fetch_all(pool)
            .await?;

            let comments_count: (i64,) = sqlx::query_as(
                "SELECT COUNT(*) FROM comments WHERE card_id = $1",
            )
            .bind(&card.id)
            .fetch_one(pool)
            .await?;

            let checklists: Vec<crate::models::checklist::TaskList> = sqlx::query_as(
                "SELECT * FROM task_lists WHERE card_id = $1 ORDER BY position",
            )
            .bind(&card.id)
            .fetch_all(pool)
            .await?;

            let mut task_list_with_tasks: Vec<crate::models::checklist::TaskListWithTasks> = Vec::new();
            for tl in checklists {
                let tasks: Vec<crate::models::checklist::Task> = sqlx::query_as(
                    "SELECT * FROM tasks WHERE task_list_id = $1 ORDER BY position",
                )
                .bind(&tl.id)
                .fetch_all(pool)
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
                start_date: card.start_date,
                due_date: card.due_date,
                is_due_completed: card.is_due_completed,
                is_closed: card.is_closed,
                created_by: card.created_by,
                members: card_members,
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

    Ok((board, list_with_cards, members))
}


pub async fn get_full_board_by_slug(pool: &sqlx::PgPool, slug: &str) -> Result<(Board, Vec<ListWithCards>, Vec<UserResponse>), AppError> {
    let board = get_by_slug(pool, slug).await?
        .ok_or(AppError::NotFound("board not found".into()))?;
    get_full_board(pool, &board.id).await
}