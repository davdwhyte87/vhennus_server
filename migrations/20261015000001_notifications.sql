-- In-app notification inbox. Each row is shown in the bell center;
-- push delivery (FCM) is best-effort on top of the stored row.
CREATE TABLE IF NOT EXISTS notifications(
    id VARCHAR(50) PRIMARY KEY,
    user_name VARCHAR(50) NOT NULL REFERENCES profiles(user_name) ON DELETE CASCADE,
    kind VARCHAR(40) NOT NULL,
    title VARCHAR(140) NOT NULL,
    body VARCHAR(500) NOT NULL,
    link VARCHAR(280),
    is_read BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_notifications_user_created ON notifications(user_name, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_notifications_user_unread ON notifications(user_name, is_read) WHERE is_read = FALSE;
