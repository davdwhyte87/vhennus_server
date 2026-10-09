-- Per-user chat read receipts (powers unread badges)

CREATE TABLE IF NOT EXISTS chat_reads (
    pair_id VARCHAR(50) NOT NULL REFERENCES chat_pairs(id) ON DELETE CASCADE,
    user_name VARCHAR(50) NOT NULL REFERENCES profiles(user_name) ON DELETE CASCADE,
    last_read_at TIMESTAMP NOT NULL DEFAULT NOW(),
    PRIMARY KEY (pair_id, user_name)
);
