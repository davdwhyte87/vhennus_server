-- Membership application pause switch (single-row settings)

CREATE TABLE IF NOT EXISTS membership_settings (
    id INTEGER PRIMARY KEY DEFAULT 1,
    applications_paused BOOLEAN NOT NULL DEFAULT FALSE,
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    CONSTRAINT single_settings_row CHECK (id = 1)
);

INSERT INTO membership_settings (id, applications_paused)
VALUES (1, FALSE)
ON CONFLICT (id) DO NOTHING;
