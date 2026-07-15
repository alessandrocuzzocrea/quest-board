-- Change attachment size column from INTEGER to BIGINT to match Rust i64
ALTER TABLE attachments ALTER COLUMN size TYPE BIGINT;
