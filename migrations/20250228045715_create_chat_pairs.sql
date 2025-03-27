-- Add migration script here

CREATE TABLE chat_pairs(
    id VARCHAR(50)  PRIMARY KEY ,
    user1 VARCHAR(50) NOT NULL ,
    user2 VARCHAR(50) NOT NULL ,
    last_message TEXT,
    all_read BOOLEAN DEFAULT FALSE NOT NULL,
    created_at TIMESTAMP DEFAULT NOW() NOT NULL ,
    updated_at TIMESTAMP DEFAULT NOW() NOT NULL
)