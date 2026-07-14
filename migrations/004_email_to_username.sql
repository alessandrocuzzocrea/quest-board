-- Replace email with username as the user login identifier

-- Add username column
ALTER TABLE users ADD COLUMN username TEXT;

-- Backfill: copy email as username
UPDATE users SET username = email;

-- Make username NOT NULL UNIQUE
ALTER TABLE users ALTER COLUMN username SET NOT NULL;
ALTER TABLE users ADD CONSTRAINT users_username_key UNIQUE (username);

-- Drop email column
ALTER TABLE users DROP COLUMN email;
