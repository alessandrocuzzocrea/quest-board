CREATE TABLE IF NOT EXISTS sessions (
    id TEXT PRIMARY KEY,
    data JSONB NOT NULL DEFAULT '{}',
    expiry_date TIMESTAMPTZ NOT NULL DEFAULT (NOW() + INTERVAL '7 days')
);

CREATE TABLE IF NOT EXISTS users (
    id TEXT PRIMARY KEY,
    email TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    name TEXT NOT NULL,
    role TEXT NOT NULL DEFAULT 'user',
    created_at TEXT NOT NULL DEFAULT (NOW()),
    updated_at TEXT NOT NULL DEFAULT (NOW())
);

CREATE TABLE IF NOT EXISTS boards (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    position DOUBLE PRECISION NOT NULL DEFAULT 0,
    created_by TEXT NOT NULL REFERENCES users(id),
    created_at TEXT NOT NULL DEFAULT (NOW()),
    updated_at TEXT NOT NULL DEFAULT (NOW())
);

CREATE TABLE IF NOT EXISTS board_members (
    id TEXT PRIMARY KEY,
    board_id TEXT NOT NULL REFERENCES boards(id) ON DELETE CASCADE,
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role TEXT NOT NULL DEFAULT 'editor',
    created_at TEXT NOT NULL DEFAULT (NOW()),
    UNIQUE(board_id, user_id)
);

CREATE TABLE IF NOT EXISTS lists (
    id TEXT PRIMARY KEY,
    board_id TEXT NOT NULL REFERENCES boards(id) ON DELETE CASCADE,
    name TEXT,
    position DOUBLE PRECISION NOT NULL DEFAULT 0,
    list_type TEXT NOT NULL DEFAULT 'active',
    color TEXT,
    created_at TEXT NOT NULL DEFAULT (NOW()),
    updated_at TEXT NOT NULL DEFAULT (NOW())
);

CREATE TABLE IF NOT EXISTS cards (
    id TEXT PRIMARY KEY,
    board_id TEXT NOT NULL REFERENCES boards(id) ON DELETE CASCADE,
    list_id TEXT NOT NULL REFERENCES lists(id) ON DELETE CASCADE,
    position DOUBLE PRECISION NOT NULL DEFAULT 0,
    name TEXT NOT NULL,
    description TEXT DEFAULT '',
    due_date TEXT,
    is_due_completed BOOLEAN NOT NULL DEFAULT FALSE,
    is_closed BOOLEAN NOT NULL DEFAULT FALSE,
    created_by TEXT NOT NULL REFERENCES users(id),
    created_at TEXT NOT NULL DEFAULT (NOW()),
    updated_at TEXT NOT NULL DEFAULT (NOW())
);

CREATE TABLE IF NOT EXISTS card_members (
    id TEXT PRIMARY KEY,
    card_id TEXT NOT NULL REFERENCES cards(id) ON DELETE CASCADE,
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at TEXT NOT NULL DEFAULT (NOW()),
    UNIQUE(card_id, user_id)
);

CREATE TABLE IF NOT EXISTS labels (
    id TEXT PRIMARY KEY,
    board_id TEXT NOT NULL REFERENCES boards(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    color TEXT NOT NULL DEFAULT '#0079bf',
    position DOUBLE PRECISION NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (NOW()),
    updated_at TEXT NOT NULL DEFAULT (NOW())
);

CREATE TABLE IF NOT EXISTS card_labels (
    id TEXT PRIMARY KEY,
    card_id TEXT NOT NULL REFERENCES cards(id) ON DELETE CASCADE,
    label_id TEXT NOT NULL REFERENCES labels(id) ON DELETE CASCADE,
    created_at TEXT NOT NULL DEFAULT (NOW()),
    UNIQUE(card_id, label_id)
);

CREATE TABLE IF NOT EXISTS comments (
    id TEXT PRIMARY KEY,
    card_id TEXT NOT NULL REFERENCES cards(id) ON DELETE CASCADE,
    user_id TEXT NOT NULL REFERENCES users(id),
    text TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (NOW()),
    updated_at TEXT NOT NULL DEFAULT (NOW())
);

CREATE TABLE IF NOT EXISTS attachments (
    id TEXT PRIMARY KEY,
    card_id TEXT NOT NULL REFERENCES cards(id) ON DELETE CASCADE,
    user_id TEXT NOT NULL REFERENCES users(id),
    name TEXT NOT NULL,
    attachment_type TEXT NOT NULL,
    file_path TEXT,
    link_url TEXT,
    size INTEGER,
    mime_type TEXT,
    created_at TEXT NOT NULL DEFAULT (NOW())
);

CREATE TABLE IF NOT EXISTS task_lists (
    id TEXT PRIMARY KEY,
    card_id TEXT NOT NULL REFERENCES cards(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    position DOUBLE PRECISION NOT NULL DEFAULT 0,
    hide_completed BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TEXT NOT NULL DEFAULT (NOW()),
    updated_at TEXT NOT NULL DEFAULT (NOW())
);

CREATE TABLE IF NOT EXISTS tasks (
    id TEXT PRIMARY KEY,
    task_list_id TEXT NOT NULL REFERENCES task_lists(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    position DOUBLE PRECISION NOT NULL DEFAULT 0,
    is_completed BOOLEAN NOT NULL DEFAULT FALSE,
    assignee_id TEXT REFERENCES users(id),
    created_at TEXT NOT NULL DEFAULT (NOW()),
    updated_at TEXT NOT NULL DEFAULT (NOW())
);

CREATE TABLE IF NOT EXISTS actions (
    id TEXT PRIMARY KEY,
    card_id TEXT NOT NULL,
    board_id TEXT,
    user_id TEXT,
    action_type TEXT NOT NULL,
    data TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL DEFAULT (NOW())
);

CREATE TABLE IF NOT EXISTS notifications (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    notif_type TEXT NOT NULL,
    data TEXT NOT NULL DEFAULT '{}',
    is_read BOOLEAN NOT NULL DEFAULT FALSE,
    card_id TEXT,
    action_id TEXT,
    created_at TEXT NOT NULL DEFAULT (NOW())
);

CREATE TABLE IF NOT EXISTS favorites (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    board_id TEXT REFERENCES boards(id) ON DELETE CASCADE,
    card_id TEXT REFERENCES cards(id) ON DELETE CASCADE,
    created_at TEXT NOT NULL DEFAULT (NOW()),
    CHECK (board_id IS NOT NULL OR card_id IS NOT NULL)
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_favorites_board ON favorites(user_id, board_id) WHERE board_id IS NOT NULL;
CREATE UNIQUE INDEX IF NOT EXISTS idx_favorites_card ON favorites(user_id, card_id) WHERE card_id IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_boards_created_by ON boards(created_by);
CREATE INDEX IF NOT EXISTS idx_cards_board_id ON cards(board_id);
CREATE INDEX IF NOT EXISTS idx_cards_list_id ON cards(list_id);
CREATE INDEX IF NOT EXISTS idx_lists_board_id ON lists(board_id);
CREATE INDEX IF NOT EXISTS idx_labels_board_id ON labels(board_id);
CREATE INDEX IF NOT EXISTS idx_card_members_card ON card_members(card_id);
CREATE INDEX IF NOT EXISTS idx_comments_card ON comments(card_id);
CREATE INDEX IF NOT EXISTS idx_attachments_card ON attachments(card_id);
CREATE INDEX IF NOT EXISTS idx_task_lists_card ON task_lists(card_id);
CREATE INDEX IF NOT EXISTS idx_tasks_list ON tasks(task_list_id);
CREATE INDEX IF NOT EXISTS idx_actions_card ON actions(card_id);
CREATE INDEX IF NOT EXISTS idx_notifications_user ON notifications(user_id);
CREATE INDEX IF NOT EXISTS idx_cards_name ON cards(name);
