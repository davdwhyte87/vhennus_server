-- Your SQL goes here
CREATE TABLE posts(
    id VARCHAR PRIMARY KEY ,
    text TEXT NOT NULL ,
    image VARCHAR,
    user_name VARCHAR REFERENCES profiles(user_name) NOT NULL,
    created_at TIMESTAMP DEFAULT NOW() NOT NULL,
    updated_at TIMESTAMP DEFAULT  NOW() NOT NULL
)