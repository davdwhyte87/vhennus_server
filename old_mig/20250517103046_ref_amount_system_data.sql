-- Add migration script here
ALTER TABLE system_data ADD COLUMN ref_amount DECIMAL(24, 3) NOT NULL DEFAULT 0.00;
