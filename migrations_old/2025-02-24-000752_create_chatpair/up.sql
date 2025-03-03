-- Your SQL goes here

CREATE TABLE chat_pairs(
    id VARCHAR PRIMARY KEY ,
    user1 VARCHAR NOT NULL ,
    user2 VARCHAR NOT NULL ,
    last_message VARCHAR,
    all_read BOOLEAN DEFAULT FALSE NOT NULL, 
    created_at TIMESTAMP DEFAULT NOW() NOT NULL ,
    updated_at TIMESTAMP DEFAULT NOW() NOT NULL 
)