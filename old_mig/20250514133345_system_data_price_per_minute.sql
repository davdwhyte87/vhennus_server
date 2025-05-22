-- Add migration script here

ALTER TABLE system_data ADD COLUMN price_per_min DECIMAL(24, 3) NOT NULL DEFAULT 0.00;
