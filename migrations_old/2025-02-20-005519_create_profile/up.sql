-- Your SQL goes here

CREATE TABLE profiles (
    id VARCHAR PRIMARY KEY,
    user_name VARCHAR UNIQUE NOT NULL,
    bio VARCHAR,
    name VARCHAR,
    image VARCHAR,
    created_at TIMESTAMP DEFAULT NOW() NOT NULL ,
    updated_at TIMESTAMP DEFAULT NOW() NOT NULL ,
    app_f_token VARCHAR
);
