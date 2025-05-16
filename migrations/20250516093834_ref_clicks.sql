-- Add migration script here
CREATE TABLE ref_clicks
(
    click_id        VARCHAR(50) PRIMARY KEY,
    code VARCHAR(50) UNIQUE NOT NULL,
    created_at TIMESTAMP DEFAULT NOW() NOT NULL
)