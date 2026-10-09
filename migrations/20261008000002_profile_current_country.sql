-- Membership application extras: current country of residence on profiles

ALTER TABLE profiles ADD COLUMN IF NOT EXISTS current_country TEXT;
