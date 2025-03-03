-- Your SQL goes here
CREATE TABLE users (
    id VARCHAR PRIMARY KEY,
    user_name VARCHAR UNIQUE NOT NULL,
    email VARCHAR(255) UNIQUE CHECK (email ~* '^[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}$') ,
    code INTEGER,
    created_at TIMESTAMP DEFAULT NOW() NOT NULL ,
    user_type VARCHAR(20) NOT NULL DEFAULT 'USER' CHECK (user_type IN ('USER', 'ADMIN')),
    password_hash TEXT NOT NULL,
    is_deleted BOOLEAN DEFAULT FALSE NOT NULL
);
