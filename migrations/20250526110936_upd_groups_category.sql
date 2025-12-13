-- Add migration script here

ALTER TABLE groups
ALTER COLUMN category TYPE TEXT[]
USING ARRAY[category];
