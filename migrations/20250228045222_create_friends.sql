-- Add migration script here
CREATE TABLE friends(
    id              SERIAL PRIMARY KEY,
    user_username   VARCHAR(50) NOT NULL REFERENCES profiles (user_name) ON DELETE CASCADE,
    friend_username VARCHAR(50) NOT NULL REFERENCES profiles (user_name) ON DELETE CASCADE,
    UNIQUE (user_username, friend_username) -- Ensure unique friendships
)