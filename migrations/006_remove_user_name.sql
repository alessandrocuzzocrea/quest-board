-- Remove display name from users — username is sufficient
ALTER TABLE users DROP COLUMN IF EXISTS name;
