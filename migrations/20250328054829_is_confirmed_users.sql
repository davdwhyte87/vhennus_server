-- Add migration script here

ALTER TABLE users ADD COLUMN email_confirmed BOOLEAN NOT NULL DEFAULT false;


