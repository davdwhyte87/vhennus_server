-- Add migration script here
CREATE TABLE comments(
     id VARCHAR(50)  PRIMARY KEY ,
     user_name VARCHAR(50) REFERENCES profiles(user_name) ON DELETE CASCADE  NOT NULL ,
     post_id VARCHAR(50)  REFERENCES posts(id) ON DELETE CASCADE NOT NULL ,
     text TEXT NOT NULL ,
     created_at TIMESTAMP DEFAULT NOW() NOT NULL
)