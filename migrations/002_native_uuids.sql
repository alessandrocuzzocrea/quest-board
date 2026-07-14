-- Convert all TEXT primary keys to native PostgreSQL UUID
-- Uses column-swap approach to avoid PG system catalog conflicts with ALTER TYPE USING

-- Helper: alter a column from TEXT to UUID using add/drop/rename pattern
-- (avoids implicit type creation issues with ALTER COLUMN ... TYPE USING)

-- === sessions — skip (tower-sessions uses u128 as string) ===

-- === users ===
ALTER TABLE users ADD COLUMN id_new UUID DEFAULT gen_random_uuid();
UPDATE users SET id_new = id::uuid;
ALTER TABLE users DROP COLUMN id CASCADE;
ALTER TABLE users RENAME COLUMN id_new TO id;
ALTER TABLE users ADD PRIMARY KEY (id);

-- === boards ===
ALTER TABLE boards ADD COLUMN id_new UUID DEFAULT gen_random_uuid();
ALTER TABLE boards ADD COLUMN created_by_new UUID;
UPDATE boards SET id_new = id::uuid, created_by_new = created_by::uuid;
ALTER TABLE boards DROP COLUMN id CASCADE;
ALTER TABLE boards DROP COLUMN created_by CASCADE;
ALTER TABLE boards RENAME COLUMN id_new TO id;
ALTER TABLE boards RENAME COLUMN created_by_new TO created_by;
ALTER TABLE boards ADD PRIMARY KEY (id);
ALTER TABLE boards ADD FOREIGN KEY (created_by) REFERENCES users(id);

-- === board_members ===
ALTER TABLE board_members ADD COLUMN id_new UUID DEFAULT gen_random_uuid();
ALTER TABLE board_members ADD COLUMN board_id_new UUID;
ALTER TABLE board_members ADD COLUMN user_id_new UUID;
UPDATE board_members SET id_new = id::uuid, board_id_new = board_id::uuid, user_id_new = user_id::uuid;
ALTER TABLE board_members DROP COLUMN id CASCADE;
ALTER TABLE board_members DROP COLUMN board_id CASCADE;
ALTER TABLE board_members DROP COLUMN user_id CASCADE;
ALTER TABLE board_members RENAME COLUMN id_new TO id;
ALTER TABLE board_members RENAME COLUMN board_id_new TO board_id;
ALTER TABLE board_members RENAME COLUMN user_id_new TO user_id;
ALTER TABLE board_members ADD PRIMARY KEY (id);
ALTER TABLE board_members ADD FOREIGN KEY (board_id) REFERENCES boards(id) ON DELETE CASCADE;
ALTER TABLE board_members ADD FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE;

-- === lists ===
ALTER TABLE lists ADD COLUMN id_new UUID DEFAULT gen_random_uuid();
ALTER TABLE lists ADD COLUMN board_id_new UUID;
UPDATE lists SET id_new = id::uuid, board_id_new = board_id::uuid;
ALTER TABLE lists DROP COLUMN id CASCADE;
ALTER TABLE lists DROP COLUMN board_id CASCADE;
ALTER TABLE lists RENAME COLUMN id_new TO id;
ALTER TABLE lists RENAME COLUMN board_id_new TO board_id;
ALTER TABLE lists ADD PRIMARY KEY (id);
ALTER TABLE lists ADD FOREIGN KEY (board_id) REFERENCES boards(id) ON DELETE CASCADE;

-- === cards ===
ALTER TABLE cards ADD COLUMN id_new UUID DEFAULT gen_random_uuid();
ALTER TABLE cards ADD COLUMN board_id_new UUID;
ALTER TABLE cards ADD COLUMN list_id_new UUID;
ALTER TABLE cards ADD COLUMN created_by_new UUID;
UPDATE cards SET id_new = id::uuid, board_id_new = board_id::uuid, list_id_new = list_id::uuid, created_by_new = created_by::uuid;
ALTER TABLE cards DROP COLUMN id CASCADE;
ALTER TABLE cards DROP COLUMN board_id CASCADE;
ALTER TABLE cards DROP COLUMN list_id CASCADE;
ALTER TABLE cards DROP COLUMN created_by CASCADE;
ALTER TABLE cards RENAME COLUMN id_new TO id;
ALTER TABLE cards RENAME COLUMN board_id_new TO board_id;
ALTER TABLE cards RENAME COLUMN list_id_new TO list_id;
ALTER TABLE cards RENAME COLUMN created_by_new TO created_by;
ALTER TABLE cards ADD PRIMARY KEY (id);
ALTER TABLE cards ADD FOREIGN KEY (board_id) REFERENCES boards(id) ON DELETE CASCADE;
ALTER TABLE cards ADD FOREIGN KEY (list_id) REFERENCES lists(id) ON DELETE CASCADE;
ALTER TABLE cards ADD FOREIGN KEY (created_by) REFERENCES users(id);

-- === card_members ===
ALTER TABLE card_members ADD COLUMN id_new UUID DEFAULT gen_random_uuid();
ALTER TABLE card_members ADD COLUMN card_id_new UUID;
ALTER TABLE card_members ADD COLUMN user_id_new UUID;
UPDATE card_members SET id_new = id::uuid, card_id_new = card_id::uuid, user_id_new = user_id::uuid;
ALTER TABLE card_members DROP COLUMN id CASCADE;
ALTER TABLE card_members DROP COLUMN card_id CASCADE;
ALTER TABLE card_members DROP COLUMN user_id CASCADE;
ALTER TABLE card_members RENAME COLUMN id_new TO id;
ALTER TABLE card_members RENAME COLUMN card_id_new TO card_id;
ALTER TABLE card_members RENAME COLUMN user_id_new TO user_id;
ALTER TABLE card_members ADD PRIMARY KEY (id);
ALTER TABLE card_members ADD FOREIGN KEY (card_id) REFERENCES cards(id) ON DELETE CASCADE;
ALTER TABLE card_members ADD FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE;

-- === labels ===
ALTER TABLE labels ADD COLUMN id_new UUID DEFAULT gen_random_uuid();
ALTER TABLE labels ADD COLUMN board_id_new UUID;
UPDATE labels SET id_new = id::uuid, board_id_new = board_id::uuid;
ALTER TABLE labels DROP COLUMN id CASCADE;
ALTER TABLE labels DROP COLUMN board_id CASCADE;
ALTER TABLE labels RENAME COLUMN id_new TO id;
ALTER TABLE labels RENAME COLUMN board_id_new TO board_id;
ALTER TABLE labels ADD PRIMARY KEY (id);
ALTER TABLE labels ADD FOREIGN KEY (board_id) REFERENCES boards(id) ON DELETE CASCADE;

-- === card_labels ===
ALTER TABLE card_labels ADD COLUMN id_new UUID DEFAULT gen_random_uuid();
ALTER TABLE card_labels ADD COLUMN card_id_new UUID;
ALTER TABLE card_labels ADD COLUMN label_id_new UUID;
UPDATE card_labels SET id_new = id::uuid, card_id_new = card_id::uuid, label_id_new = label_id::uuid;
ALTER TABLE card_labels DROP COLUMN id CASCADE;
ALTER TABLE card_labels DROP COLUMN card_id CASCADE;
ALTER TABLE card_labels DROP COLUMN label_id CASCADE;
ALTER TABLE card_labels RENAME COLUMN id_new TO id;
ALTER TABLE card_labels RENAME COLUMN card_id_new TO card_id;
ALTER TABLE card_labels RENAME COLUMN label_id_new TO label_id;
ALTER TABLE card_labels ADD PRIMARY KEY (id);
ALTER TABLE card_labels ADD FOREIGN KEY (card_id) REFERENCES cards(id) ON DELETE CASCADE;
ALTER TABLE card_labels ADD FOREIGN KEY (label_id) REFERENCES labels(id) ON DELETE CASCADE;

-- === comments ===
ALTER TABLE comments ADD COLUMN id_new UUID DEFAULT gen_random_uuid();
ALTER TABLE comments ADD COLUMN card_id_new UUID;
ALTER TABLE comments ADD COLUMN user_id_new UUID;
UPDATE comments SET id_new = id::uuid, card_id_new = card_id::uuid, user_id_new = user_id::uuid;
ALTER TABLE comments DROP COLUMN id CASCADE;
ALTER TABLE comments DROP COLUMN card_id CASCADE;
ALTER TABLE comments DROP COLUMN user_id CASCADE;
ALTER TABLE comments RENAME COLUMN id_new TO id;
ALTER TABLE comments RENAME COLUMN card_id_new TO card_id;
ALTER TABLE comments RENAME COLUMN user_id_new TO user_id;
ALTER TABLE comments ADD PRIMARY KEY (id);
ALTER TABLE comments ADD FOREIGN KEY (card_id) REFERENCES cards(id) ON DELETE CASCADE;
ALTER TABLE comments ADD FOREIGN KEY (user_id) REFERENCES users(id);

-- === attachments ===
ALTER TABLE attachments ADD COLUMN id_new UUID DEFAULT gen_random_uuid();
ALTER TABLE attachments ADD COLUMN card_id_new UUID;
ALTER TABLE attachments ADD COLUMN user_id_new UUID;
UPDATE attachments SET id_new = id::uuid, card_id_new = card_id::uuid, user_id_new = user_id::uuid;
ALTER TABLE attachments DROP COLUMN id CASCADE;
ALTER TABLE attachments DROP COLUMN card_id CASCADE;
ALTER TABLE attachments DROP COLUMN user_id CASCADE;
ALTER TABLE attachments RENAME COLUMN id_new TO id;
ALTER TABLE attachments RENAME COLUMN card_id_new TO card_id;
ALTER TABLE attachments RENAME COLUMN user_id_new TO user_id;
ALTER TABLE attachments ADD PRIMARY KEY (id);
ALTER TABLE attachments ADD FOREIGN KEY (card_id) REFERENCES cards(id) ON DELETE CASCADE;
ALTER TABLE attachments ADD FOREIGN KEY (user_id) REFERENCES users(id);

-- === task_lists ===
ALTER TABLE task_lists ADD COLUMN id_new UUID DEFAULT gen_random_uuid();
ALTER TABLE task_lists ADD COLUMN card_id_new UUID;
UPDATE task_lists SET id_new = id::uuid, card_id_new = card_id::uuid;
ALTER TABLE task_lists DROP COLUMN id CASCADE;
ALTER TABLE task_lists DROP COLUMN card_id CASCADE;
ALTER TABLE task_lists RENAME COLUMN id_new TO id;
ALTER TABLE task_lists RENAME COLUMN card_id_new TO card_id;
ALTER TABLE task_lists ADD PRIMARY KEY (id);
ALTER TABLE task_lists ADD FOREIGN KEY (card_id) REFERENCES cards(id) ON DELETE CASCADE;

-- === tasks ===
ALTER TABLE tasks ADD COLUMN id_new UUID DEFAULT gen_random_uuid();
ALTER TABLE tasks ADD COLUMN task_list_id_new UUID;
ALTER TABLE tasks ADD COLUMN assignee_id_new UUID;
UPDATE tasks SET id_new = id::uuid, task_list_id_new = task_list_id::uuid, assignee_id_new = assignee_id::uuid;
ALTER TABLE tasks DROP COLUMN id CASCADE;
ALTER TABLE tasks DROP COLUMN task_list_id CASCADE;
ALTER TABLE tasks DROP COLUMN assignee_id CASCADE;
ALTER TABLE tasks RENAME COLUMN id_new TO id;
ALTER TABLE tasks RENAME COLUMN task_list_id_new TO task_list_id;
ALTER TABLE tasks RENAME COLUMN assignee_id_new TO assignee_id;
ALTER TABLE tasks ADD PRIMARY KEY (id);
ALTER TABLE tasks ADD FOREIGN KEY (task_list_id) REFERENCES task_lists(id) ON DELETE CASCADE;
ALTER TABLE tasks ADD FOREIGN KEY (assignee_id) REFERENCES users(id);

-- === actions ===
ALTER TABLE actions ADD COLUMN id_new UUID DEFAULT gen_random_uuid();
ALTER TABLE actions ADD COLUMN card_id_new UUID;
ALTER TABLE actions ADD COLUMN board_id_new UUID;
ALTER TABLE actions ADD COLUMN user_id_new UUID;
UPDATE actions SET id_new = id::uuid, card_id_new = card_id::uuid, board_id_new = board_id::uuid, user_id_new = user_id::uuid;
ALTER TABLE actions DROP COLUMN id CASCADE;
ALTER TABLE actions DROP COLUMN card_id CASCADE;
ALTER TABLE actions DROP COLUMN board_id CASCADE;
ALTER TABLE actions DROP COLUMN user_id CASCADE;
ALTER TABLE actions RENAME COLUMN id_new TO id;
ALTER TABLE actions RENAME COLUMN card_id_new TO card_id;
ALTER TABLE actions RENAME COLUMN board_id_new TO board_id;
ALTER TABLE actions RENAME COLUMN user_id_new TO user_id;
ALTER TABLE actions ADD PRIMARY KEY (id);

-- === notifications ===
ALTER TABLE notifications ADD COLUMN id_new UUID DEFAULT gen_random_uuid();
ALTER TABLE notifications ADD COLUMN user_id_new UUID;
ALTER TABLE notifications ADD COLUMN card_id_new UUID;
ALTER TABLE notifications ADD COLUMN action_id_new UUID;
UPDATE notifications SET id_new = id::uuid, user_id_new = user_id::uuid, card_id_new = card_id::uuid, action_id_new = action_id::uuid;
ALTER TABLE notifications DROP COLUMN id CASCADE;
ALTER TABLE notifications DROP COLUMN user_id CASCADE;
ALTER TABLE notifications DROP COLUMN card_id CASCADE;
ALTER TABLE notifications DROP COLUMN action_id CASCADE;
ALTER TABLE notifications RENAME COLUMN id_new TO id;
ALTER TABLE notifications RENAME COLUMN user_id_new TO user_id;
ALTER TABLE notifications RENAME COLUMN card_id_new TO card_id;
ALTER TABLE notifications RENAME COLUMN action_id_new TO action_id;
ALTER TABLE notifications ADD PRIMARY KEY (id);
ALTER TABLE notifications ADD FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE;

-- === favorites ===
ALTER TABLE favorites ADD COLUMN id_new UUID DEFAULT gen_random_uuid();
ALTER TABLE favorites ADD COLUMN user_id_new UUID;
ALTER TABLE favorites ADD COLUMN board_id_new UUID;
ALTER TABLE favorites ADD COLUMN card_id_new UUID;
UPDATE favorites SET id_new = id::uuid, user_id_new = user_id::uuid, board_id_new = board_id::uuid, card_id_new = card_id::uuid;
ALTER TABLE favorites DROP COLUMN id CASCADE;
ALTER TABLE favorites DROP COLUMN user_id CASCADE;
ALTER TABLE favorites DROP COLUMN board_id CASCADE;
ALTER TABLE favorites DROP COLUMN card_id CASCADE;
ALTER TABLE favorites RENAME COLUMN id_new TO id;
ALTER TABLE favorites RENAME COLUMN user_id_new TO user_id;
ALTER TABLE favorites RENAME COLUMN board_id_new TO board_id;
ALTER TABLE favorites RENAME COLUMN card_id_new TO card_id;
ALTER TABLE favorites ADD PRIMARY KEY (id);
ALTER TABLE favorites ADD FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE;
ALTER TABLE favorites ADD FOREIGN KEY (board_id) REFERENCES boards(id) ON DELETE CASCADE;
ALTER TABLE favorites ADD FOREIGN KEY (card_id) REFERENCES cards(id) ON DELETE CASCADE;

-- === Add slug column to boards ===
ALTER TABLE boards ADD COLUMN IF NOT EXISTS slug TEXT;
UPDATE boards SET slug = substr(replace(gen_random_uuid()::text, '-', ''), 1, 8);
ALTER TABLE boards ALTER COLUMN slug SET NOT NULL;
CREATE UNIQUE INDEX IF NOT EXISTS idx_boards_slug ON boards(slug);

-- === Recreate indexes that were dropped with CASCADE ===
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
