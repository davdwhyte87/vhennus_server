-- Polls attached to posts. One poll per post (set of options), one vote per user per post.
CREATE TABLE IF NOT EXISTS poll_options(
    id VARCHAR(50) PRIMARY KEY,
    post_id VARCHAR(50) NOT NULL REFERENCES posts(id) ON DELETE CASCADE,
    text VARCHAR(280) NOT NULL,
    position INT NOT NULL DEFAULT 0,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_poll_options_post ON poll_options(post_id);

CREATE TABLE IF NOT EXISTS poll_votes(
    id VARCHAR(50) PRIMARY KEY,
    post_id VARCHAR(50) NOT NULL REFERENCES posts(id) ON DELETE CASCADE,
    option_id VARCHAR(50) NOT NULL REFERENCES poll_options(id) ON DELETE CASCADE,
    user_name VARCHAR(50) NOT NULL REFERENCES profiles(user_name) ON DELETE CASCADE,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    UNIQUE(post_id, user_name)
);
CREATE INDEX IF NOT EXISTS idx_poll_votes_post ON poll_votes(post_id);
CREATE INDEX IF NOT EXISTS idx_poll_votes_option ON poll_votes(option_id);
CREATE INDEX IF NOT EXISTS idx_poll_votes_user ON poll_votes(user_name);
