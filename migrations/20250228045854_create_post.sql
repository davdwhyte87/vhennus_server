-- Add migration script here
CREATE TABLE posts(
    id VARCHAR(50)  PRIMARY KEY ,
    text TEXT NOT NULL ,
    image TEXT,
    user_name VARCHAR(50) REFERENCES profiles(user_name) NOT NULL,
    created_at TIMESTAMP DEFAULT NOW() NOT NULL,
    updated_at TIMESTAMP DEFAULT  NOW() NOT NULL
)