-- Add migration script here
CREATE TABLE likes(
    user_name VARCHAR(50) REFERENCES profiles(user_name) ON DELETE CASCADE NOT NULL ,
    post_id VARCHAR(50)  REFERENCES  posts(id) ON DELETE CASCADE NOT NULL ,
    PRIMARY KEY (user_name, post_id)

);