-- Skills & interests for friend search.
-- Admin-managed tags users can attach to their profile and filter by.

CREATE TABLE IF NOT EXISTS search_tags(
  id VARCHAR PRIMARY KEY,
  name VARCHAR(80) NOT NULL,
  kind VARCHAR(20) NOT NULL DEFAULT 'skill' CHECK (kind IN ('skill', 'interest')),
  created_at TIMESTAMP NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
  UNIQUE(name, kind)
);
CREATE INDEX IF NOT EXISTS idx_search_tags_kind ON search_tags(kind);
CREATE INDEX IF NOT EXISTS idx_search_tags_name ON search_tags(name);

CREATE TABLE IF NOT EXISTS profile_search_tags(
  profile_user_name VARCHAR(50) NOT NULL REFERENCES profiles(user_name) ON DELETE CASCADE,
  tag_id VARCHAR NOT NULL REFERENCES search_tags(id) ON DELETE CASCADE,
  created_at TIMESTAMP NOT NULL DEFAULT NOW(),
  PRIMARY KEY (profile_user_name, tag_id)
);
CREATE INDEX IF NOT EXISTS idx_profile_search_tags_user ON profile_search_tags(profile_user_name);
CREATE INDEX IF NOT EXISTS idx_profile_search_tags_tag ON profile_search_tags(tag_id);

-- Seed a small starter set so search filters are useful out of the box.
INSERT INTO search_tags(id, name, kind) VALUES
  ('skill-photography','photography','skill'),
  ('skill-coding','coding','skill'),
  ('skill-design','design','skill'),
  ('skill-cooking','cooking','skill'),
  ('skill-music-production','music production','skill'),
  ('skill-writing','writing','skill'),
  ('interest-football','football','interest'),
  ('interest-music','music','interest'),
  ('interest-movies','movies','interest'),
  ('interest-travel','travel','interest'),
  ('interest-gaming','gaming','interest'),
  ('interest-reading','reading','interest')
ON CONFLICT (name, kind) DO NOTHING;
