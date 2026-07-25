# quest-board

Kanban board with a Rust API backend (Axum + PostgreSQL) and React SPA frontend (Vite + TypeScript).

## Architecture

```
quest-board/
├── app/                # React SPA (Vite + TypeScript)
│   ├── src/            # Components, API client, tests
│   └── dist/           # Production build
├── src/                # Rust API backend
│   ├── main.rs         # Server startup + routing
│   ├── db/             # Migration runner + admin seed
│   ├── error.rs        # Error types
│   ├── models/         # Domain types (shared with frontend via ts-rs)
│   ├── handlers/       # HTTP handlers (thin — call services)
│   └── repository/     # Data access layer (all SQL here)
├── migrations/         # SQL schema
├── tests/              # Rust integration tests
└── bindings/           # Generated TypeScript bindings from Rust types
```

## Prerequisites

- Rust 1.85+
- PostgreSQL 17 (or Docker)

## Dev container (VS Code / Codespaces)

Open the repo in a Codespace or re-open in container:

```sh
# The devcontainer sets up:
#   - Rust + cargo, Bun, PostgreSQL (Docker)
#   - VS Code extensions (rust-analyzer, ESLint, Prettier, Tailwind)
#   - Port forwarding: 3001 (backend), 5173 (frontend dev server)
#
# After the container starts:
cd app && bun run dev   # frontend hot-reload
# In another terminal:
cargo run               # backend API
```
- [Bun](https://bun.sh) 1.3+

## Quick start

```sh
# 1. Start PostgreSQL
docker run -d --name quest-pg \
  -e POSTGRES_PASSWORD=quest \
  -e POSTGRES_DB=quest \
  -p 5432:5432 \
  postgres:17

# 2. Set up environment
cp .env.example .env
# Edit .env and set a random APP_SECRET

# 3. Build frontend
cd app && bun install && bun run build && cd ..

# 4. Start the backend
cargo run
# App at http://localhost:3001
```

The Rust binary embeds the built SPA via `include_str!`, so step 3 must run before step 4 (or after any frontend change).

## Development

### Frontend dev server (with hot reload)

```sh
cd app
bun run dev
# Vite dev server at http://localhost:5173
# Proxies /api/* to http://localhost:3001
```

Run the Rust backend in one terminal and Vite in another. The Vite dev server auto-reloads on file changes.

### Backend auto-reload

```sh
cargo install cargo-watch
cargo dev    # alias for cargo watch -w src -x run
```

### Frontend tests

```sh
cd app && bun run test
```

39 tests across API client, auth page, nav bar, and gantt chart components.

### Rust tests

```sh
cargo test --lib          # unit tests (no DB needed)
cargo test                # integration tests (PostgreSQL required)
```

## Production build

```sh
cd app && bun run build                      # build SPA to app/dist/
DATABASE_URL="..." cargo build --release     # embeds SPA in binary
./target/release/quest-board
```

## Docker

```sh
docker build -t quest-board .
```

Multi-stage build: `oven/bun` builds the frontend, then `rust:slim-bookworm` compiles the backend.

## API

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
| `/api/v1/events` | GET | SSE stream for real-time updates |

## Tech stack

- **Backend**: Rust, Axum, SQLx (PostgreSQL), tower-sessions
- **Frontend**: React 19, TypeScript, Vite, @dnd-kit (drag & drop), react-router-dom
- **Real-time**: Server-Sent Events (SSE)
