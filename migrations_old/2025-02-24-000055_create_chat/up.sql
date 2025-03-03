-- Your SQL goes here

CREATE TABLE chats(
    id VARCHAR PRIMARY KEY ,
    sender VARCHAR NOT NULL ,
    receiver VARCHAR NOT NULL,
    message VARCHAR NOT NULL ,
    image VARCHAR,
    created_at TIMESTAMP DEFAULT NOW() NOT NULL,
    updated_at TIMESTAMP DEFAULT NOW() NOT NULL ,
    pair_id VARCHAR NOT NULL
)
