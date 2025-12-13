-- Add migration script here
CREATE TABLE rooms(
    id VARCHAR PRIMARY KEY ,
    name VARCHAR(50) NOT NULL ,
    group_id VARCHAR NOT NULL REFERENCES groups(id) ON DELETE CASCADE ,
    description VARCHAR(100),
    is_private BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE TABLE user_rooms(
    user_name VARCHAR(50) NOT NULL REFERENCES profiles(user_name) ON DELETE CASCADE ,
    room_id VARCHAR NOT NULL REFERENCES rooms(id) ON DELETE CASCADE ,
    PRIMARY KEY (user_name, room_id),
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE  TABLE room_messages(
    id VARCHAR PRIMARY KEY,
    user_name VARCHAR(50) REFERENCES profiles(user_Name) ON DELETE CASCADE  NOT NULL ,
    text TEXT NOT NULL,
    image TEXT,
    room_id VARCHAR(100) REFERENCES rooms(id),
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

ALTER TABLE rooms ADD COLUMN created_by VARCHAR NOT NULL default "";
ALTER TABLE rooms ADD COLUMN code VARCHAR;

ALTER TABLE rooms
    ADD COLUMN member_count BIGINT NOT NULL DEFAULT 0;

CREATE INDEX idx_user_rooms_room_id ON user_rooms(room_id);
CREATE INDEX idx_rooms_group_id ON rooms(group_id);