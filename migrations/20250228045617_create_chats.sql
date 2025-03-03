-- Add migration script here
CREATE TABLE chats(

    id VARCHAR(50)  PRIMARY KEY ,
    sender VARCHAR(50) NOT NULL ,
    receiver VARCHAR(50) NOT NULL,
    message TEXT NOT NULL ,
    image TEXT,
    created_at TIMESTAMP DEFAULT NOW() NOT NULL,
    updated_at TIMESTAMP DEFAULT NOW() NOT NULL ,
    pair_id VARCHAR(50)  NOT NULL
)