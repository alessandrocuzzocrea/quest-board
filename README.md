# quest-board

Kanban board API. Rust backend (Axum + PostgreSQL).

## Prerequisites

- Rust 1.85+
- PostgreSQL 17 (or Docker)

## Quick start

```sh
# 1. Start PostgreSQL
docker run -d --name quest-pg \
  -e POSTGRES_PASSWORD=quest \
  -e POSTGRES_DB=quest \
  -p 5432:5432 \
  postgres:17

# 2. Set up environment
cp backend/.env.example backend/.env
# Edit backend/.env and set a random APP_SECRET

# 3. Start the backend
cd backend
cargo run
# API at http://localhost:3001
```

## Backend auto-reload with cargo watch

The `cargo dev` alias (defined in `backend/.cargo/config.toml`) runs:

```sh
cargo watch -w src -x run
```

This recompiles and restarts the backend whenever a file in `backend/src/` changes.
Install `cargo-watch` once:

```sh
cargo install cargo-watch
```

Then:

```sh
cd backend && cargo dev
```

## Production build

```sh
cd backend
DATABASE_URL="..." cargo build --release
./target/release/quest-board
```

## Project structure

```
quest-board/
├── backend/           # Rust API
│   ├── src/
│   │   ├── main.rs          # server startup + routing
│   │   ├── db/              # migration runner
│   │   ├── error.rs         # error types
│   │   ├── models/          # domain types
│   │   ├── handlers/        # HTTP handlers (thin — call repos)
│   │   └── repository/      # data access layer (all SQL here)
│   └── migrations/          # SQL schema
└── .gitignore
```

## API overview

| Route | Methods | Description |
|---|---|---|
| `/api/v1/auth/register` | POST | Create account |
| `/api/v1/auth/login` | POST | Login (sets session cookie) |
| `/api/v1/boards` | GET, POST | List / create boards |
| `/api/v1/boards/{id}` | GET, PUT, DELETE | Board with lists + cards nested |
| `/api/v1/lists` | POST | Create list (column) |
| `/api/v1/cards` | POST | Create card |
| `/api/v1/cards/{id}` | GET, PUT, DELETE | Card detail with labels, members, comments, checklists, actions |
| `/api/v1/cards/{id}/move` | PUT | Move card between lists |
| `/api/v1/cards/{id}/members` | POST, DELETE | Assign / unassign users |
| `/api/v1/cards/{id}/labels` | POST, DELETE | Attach / detach labels |
| `/api/v1/cards/{id}/task-lists` | POST | Add checklist |
| `/api/v1/comments` | POST | Add comment |
| `/api/v1/labels` | POST | Create label |
| `/api/v1/search?q=` | GET | Search cards + boards |

Full list at `src/handlers/`.
