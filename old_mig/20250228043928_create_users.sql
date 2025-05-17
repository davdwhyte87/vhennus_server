-- Add migration script here
CREATE TABLE users(
    id VARCHAR(50)  PRIMARY KEY,
    user_name VARCHAR(50) UNIQUE NOT NULL,
    email  VARCHAR(255) UNIQUE ,
    code INTEGER,
    created_at TIMESTAMP DEFAULT NOW() NOT NULL ,
    user_type VARCHAR(20) NOT NULL DEFAULT 'USER' ,
    password_hash TEXT NOT NULL,
    is_deleted BOOLEAN DEFAULT FALSE NOT NULL,
    CONSTRAINT user_type_format CHECK (user_type IN ('USER', 'ADMIN'))
)
