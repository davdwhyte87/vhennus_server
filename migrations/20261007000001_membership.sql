-- Membership feature: profile flag + questions/options + applications/answers

ALTER TABLE profiles ADD COLUMN IF NOT EXISTS membership BOOLEAN NOT NULL DEFAULT FALSE;
ALTER TABLE profiles ADD COLUMN IF NOT EXISTS phone_number TEXT;

CREATE TABLE IF NOT EXISTS membership_questions (
    id VARCHAR(50) PRIMARY KEY,
    question TEXT NOT NULL,
    is_required BOOLEAN NOT NULL DEFAULT TRUE,
    display_order INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS membership_question_options (
    id VARCHAR(50) PRIMARY KEY,
    question_id VARCHAR(50) NOT NULL REFERENCES membership_questions(id) ON DELETE CASCADE,
    option_text TEXT NOT NULL,
    is_correct BOOLEAN NOT NULL DEFAULT FALSE,
    display_order INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS membership_applications (
    id VARCHAR(50) PRIMARY KEY,
    user_name VARCHAR(50) NOT NULL REFERENCES profiles(user_name) ON DELETE CASCADE,
    status VARCHAR(20) NOT NULL DEFAULT 'submitted',
    score INTEGER NOT NULL DEFAULT 0,
    submitted_at TIMESTAMP NOT NULL DEFAULT NOW(),
    reviewed_by VARCHAR(50),
    reviewed_at TIMESTAMP,
    decision_reason TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    CONSTRAINT membership_status_format CHECK (status IN ('submitted', 'under_review', 'approved', 'rejected'))
);

-- One pending (submitted/under_review) application per user at most
CREATE UNIQUE INDEX IF NOT EXISTS uniq_pending_membership_application
    ON membership_applications(user_name)
    WHERE status IN ('submitted', 'under_review');

CREATE TABLE IF NOT EXISTS membership_application_answers (
    id VARCHAR(50) PRIMARY KEY,
    application_id VARCHAR(50) NOT NULL REFERENCES membership_applications(id) ON DELETE CASCADE,
    question_id VARCHAR(50) NOT NULL REFERENCES membership_questions(id) ON DELETE CASCADE,
    option_id VARCHAR(50) NOT NULL REFERENCES membership_question_options(id) ON DELETE CASCADE,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    CONSTRAINT uniq_application_question UNIQUE (application_id, question_id)
);

-- Seed: sample membership questions (edit text/options via DB, is_correct stays server-side)
INSERT INTO membership_questions (id, question, is_required, display_order) VALUES
    ('mq1', 'What is the minimum age to become a member?', TRUE, 1),
    ('mq2', 'Which behaviour is expected of members?', TRUE, 2),
    ('mq3', 'Members must verify their identity when asked by an admin.', TRUE, 3),
    ('mq4', 'What should you do if you see harmful content in the community?', TRUE, 4),
    ('mq5', 'Can you share your login details with other people?', TRUE, 5)
ON CONFLICT (id) DO NOTHING;

INSERT INTO membership_question_options (id, question_id, option_text, is_correct, display_order) VALUES
    ('mq1o1', 'mq1', '13', FALSE, 1),
    ('mq1o2', 'mq1', '16', FALSE, 2),
    ('mq1o3', 'mq1', '18', TRUE, 3),
    ('mq1o4', 'mq1', '21', FALSE, 4),
    ('mq2o1', 'mq2', 'Respect other members', TRUE, 1),
    ('mq2o2', 'mq2', 'Spam the feed', FALSE, 2),
    ('mq2o3', 'mq2', 'Share other members private information', FALSE, 3),
    ('mq2o4', 'mq2', 'Harass new users', FALSE, 4),
    ('mq3o1', 'mq3', 'True', TRUE, 1),
    ('mq3o2', 'mq3', 'False', FALSE, 2),
    ('mq4o1', 'mq4', 'Report it', TRUE, 1),
    ('mq4o2', 'mq4', 'Ignore it', FALSE, 2),
    ('mq4o3', 'mq4', 'Share it with others', FALSE, 3),
    ('mq4o4', 'mq4', 'Reply with abuse', FALSE, 4),
    ('mq5o1', 'mq5', 'No', TRUE, 1),
    ('mq5o2', 'mq5', 'Yes', FALSE, 2)
ON CONFLICT (id) DO NOTHING;
