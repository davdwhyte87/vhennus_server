-- Add migration script here

CREATE TABLE groups(
    id VARCHAR PRIMARY KEY,
    user_name VARCHAR(50) NOT NULL REFERENCES profiles(user_name) ON DELETE CASCADE ,
    name VARCHAR(100) NOT NULL UNIQUE ,
    description VARCHAR(250),
    is_private BOOLEAN NOT NULL DEFAULT false,
    image TEXT,
    category VARCHAR(100),
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

ALTER TABLE groups
    ALTER COLUMN category SET NOT NULL;
