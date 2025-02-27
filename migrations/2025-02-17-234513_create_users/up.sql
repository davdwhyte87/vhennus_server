-- Your SQL goes here
CREATE TABLE users (
    id VARCHAR PRIMARY KEY,
    user_name VARCHAR UNIQUE NOT NULL,
    email VARCHAR UNIQUE ,
    code INTEGER,
    created_at TIMESTAMP DEFAULT NOW() NOT NULL ,
    user_type VARCHAR NOT NULL DEFAULT 'USER',
    password_hash TEXT NOT NULL,
    is_deleted BOOLEAN DEFAULT FALSE NOT NULL
);
