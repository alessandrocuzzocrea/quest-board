-- Replace email with username as the user login identifier.
-- Fully idempotent — safe to re-run every startup.

ALTER TABLE users ADD COLUMN IF NOT EXISTS username TEXT;
ALTER TABLE users ALTER COLUMN username SET NOT NULL;
ALTER TABLE users DROP COLUMN IF EXISTS email;

-- Add unique constraint on username (replaces the removed UNIQUE on email)
CREATE UNIQUE INDEX IF NOT EXISTS idx_users_username ON users(username);
