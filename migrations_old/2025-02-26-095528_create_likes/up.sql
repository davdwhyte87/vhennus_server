-- Your SQL goes here
CREATE TABLE likes(
    user_name VARCHAR REFERENCES profiles(user_name) ON DELETE CASCADE NOT NULL ,
    post_id VARCHAR REFERENCES  posts(id) ON DELETE CASCADE NOT NULL ,
    PRIMARY KEY (user_name, post_id)

)