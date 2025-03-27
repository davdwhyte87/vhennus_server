-- Your SQL goes here

CREATE TABLE friend_requests
(
    id VARCHAR PRIMARY KEY,
    user_name VARCHAR NOT NULL REFERENCES profiles(user_name) ON DELETE CASCADE ,
    requester VARCHAR NOT NULL REFERENCES profiles(user_name) ON DELETE CASCADE ,
    status VARCHAR NOT NULL DEFAULT 'PENDING',
    created_at TIMESTAMP DEFAULT NOW() NOT NULL ,
    updated_at TIMESTAMP DEFAULT NOW() NOT NULL ,
    UNIQUE (user_name, requester)
)
