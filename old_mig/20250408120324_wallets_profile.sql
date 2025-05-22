-- Add migration script here
ALTER TABLE profiles ADD COLUMN wallets TEXT NOT NULL DEFAULT ' ';

