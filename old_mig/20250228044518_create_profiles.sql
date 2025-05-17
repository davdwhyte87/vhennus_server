-- Add migration script here
CREATE TABLE profiles (
    id VARCHAR(50)  PRIMARY KEY,
    user_name VARCHAR(50) UNIQUE NOT NULL,
    bio VARCHAR(1000),
    name VARCHAR(100),
    image TEXT,
    created_at TIMESTAMP DEFAULT NOW() NOT NULL ,
    updated_at TIMESTAMP DEFAULT NOW() NOT NULL ,
    app_f_token TEXT
);