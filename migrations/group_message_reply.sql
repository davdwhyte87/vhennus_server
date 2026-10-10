-- Group message replies: quoted message reference (nullable, no FK so history
-- survives even if the quoted row is removed; app validates same-group).
ALTER TABLE group_messages
    ADD COLUMN IF NOT EXISTS reply_to_msg_id TEXT;
CREATE INDEX IF NOT EXISTS idx_group_messages_reply_to ON group_messages(reply_to_msg_id);
