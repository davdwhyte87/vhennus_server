-- One chat pair per unordered user pair (prevents race-created duplicates)

CREATE UNIQUE INDEX IF NOT EXISTS uniq_chat_pair_users
    ON chat_pairs (LEAST(user1, user2), GREATEST(user1, user2));
