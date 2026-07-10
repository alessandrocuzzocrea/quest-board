# quest-board

Kanban board. Rust backend (Axum + PostgreSQL), SvelteKit frontend.

## Prerequisites

- Rust 1.85+
- Node.js 20+
- PostgreSQL 17 (or Docker)

## Quick start

```sh
# 1. Start PostgreSQL
docker run -d --name quest-pg \
  -e POSTGRES_PASSWORD=quest \
  -e POSTGRES_DB=quest \
  -p 5432:5432 \
  postgres:17

# 2. Start the backend
cd backend
DATABASE_URL="postgres://postgres:quest@localhost:5432/quest" cargo run
# API at http://localhost:3001/api/v1

# 3. (another terminal) Start the frontend dev server
cd frontend
npm install
npm run dev
# App at http://localhost:5173 — proxies /api/* to the backend
```

## Production build

```sh
cd frontend
npm run build          # outputs to ../backend/static/

cd ../backend
DATABASE_URL="..." cargo build --release
./target/release/quest-board
# Single binary serves both the API and the SPA on :3001
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
│   ├── migrations/          # SQL schema
│   └── static/              # frontend build output (gitignored)
├── frontend/          # SvelteKit SPA
│   ├── src/routes/
│   └── svelte.config.js     # static adapter, outputs to ../backend/static/
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
