-- 1:1 chat replies: quoted message reference (nullable, no FK so history
-- survives even if the quoted row is removed; app validates same-pair).
ALTER TABLE chats
    ADD COLUMN IF NOT EXISTS reply_to_id TEXT;
CREATE INDEX IF NOT EXISTS idx_chats_reply_to ON chats(reply_to_id);
