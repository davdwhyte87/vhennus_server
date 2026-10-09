-- Membership application extras: origin + date of birth on profiles
-- (phone_number already exists on profiles)

ALTER TABLE profiles ADD COLUMN IF NOT EXISTS country_of_origin TEXT;
ALTER TABLE profiles ADD COLUMN IF NOT EXISTS state_of_origin TEXT;
ALTER TABLE profiles ADD COLUMN IF NOT EXISTS date_of_birth DATE;
