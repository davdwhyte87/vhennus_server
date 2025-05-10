-- Add migration script here

ALTER TABLE profiles ADD COLUMN unclaimed_earnings DECIMAL(24, 3) NOT NULL DEFAULT 0.00;
ALTER TABLE profiles ADD COLUMN is_earnings_activated BOOLEAN NOT NULL DEFAULT false;
ALTER TABLE profiles ADD COLUMN referred_users TEXT[] NOT NULL DEFAULT '{}'::text[];
ALTER TABLE profiles ADD COLUMN earnings_wallet TEXT;





