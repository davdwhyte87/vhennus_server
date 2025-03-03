-- Add migration script here
CREATE TABLE friend_requests
(
    id VARCHAR(50) PRIMARY KEY,
    user_name VARCHAR(50) NOT NULL REFERENCES profiles(user_name) ON DELETE CASCADE ,
    requester VARCHAR(50) NOT NULL REFERENCES profiles(user_name) ON DELETE CASCADE ,
    status VARCHAR(50)  NOT NULL DEFAULT 'PENDING' CHECK (friend_requests.status IN ('PENDING', 'ACCEPTED','REJECTED')),
    created_at TIMESTAMP DEFAULT NOW() NOT NULL ,
    updated_at TIMESTAMP DEFAULT NOW() NOT NULL ,
    UNIQUE (user_name, requester)
)