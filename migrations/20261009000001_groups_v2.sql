-- Groups v2: one group = one chat feed (no rooms).
-- Keeps legacy groups/rooms tables untouched; adds columns + new tables.

-- 1. Extend groups with about + invite_code
ALTER TABLE groups ADD COLUMN IF NOT EXISTS about VARCHAR(500);
ALTER TABLE groups ADD COLUMN IF NOT EXISTS invite_code VARCHAR(16);

-- Backfill about from description where null
UPDATE groups SET about = description WHERE about IS NULL;

-- Backfill invite_code for existing rows (12-char hex from md5)
UPDATE groups SET invite_code = substr(md5(id || random()::text), 1, 12)
WHERE invite_code IS NULL;

ALTER TABLE groups ALTER COLUMN invite_code SET NOT NULL;
DO $$ BEGIN
  IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'groups_invite_code_unique') THEN
    ALTER TABLE groups ADD CONSTRAINT groups_invite_code_unique UNIQUE (invite_code);
  END IF;
END $$;

-- 2. Canonical categories (admin-editable) + mapping
CREATE TABLE IF NOT EXISTS group_categories(
  id VARCHAR PRIMARY KEY,
  name VARCHAR(80) NOT NULL UNIQUE,
  created_at TIMESTAMP NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS group_category_map(
  group_id VARCHAR NOT NULL REFERENCES groups(id) ON DELETE CASCADE,
  category_id VARCHAR NOT NULL REFERENCES group_categories(id) ON DELETE CASCADE,
  PRIMARY KEY (group_id, category_id)
);

-- Seed from the old hardcoded GROUP_CATEGORIES list (lowercase, deduped)
INSERT INTO group_categories(id, name) VALUES
  ('cat-technology','technology'),('cat-science','science'),('cat-art','art'),
  ('cat-music','music'),('cat-sports','sports'),('cat-health','health'),
  ('cat-education','education'),('cat-gaming','gaming'),('cat-finance','finance'),
  ('cat-travel','travel'),('cat-news','news'),('cat-lifestyle','lifestyle'),
  ('cat-programming','programming'),('cat-movies','movies'),('cat-books','books'),
  ('cat-fashion','fashion'),('cat-food','food'),('cat-fitness','fitness'),
  ('cat-photography','photography'),('cat-history','history'),('cat-culture','culture'),
  ('cat-relationships','relationships'),('cat-parenting','parenting'),('cat-business','business'),
  ('cat-entrepreneurship','entrepreneurship'),('cat-marketing','marketing'),
  ('cat-self-improvement','self-improvement'),('cat-mental-health','mental-health'),
  ('cat-memes','memes'),('cat-crypto','crypto'),('cat-blockchain','blockchain'),
  ('cat-design','design'),('cat-productivity','productivity'),('cat-spirituality','spirituality'),
  ('cat-philosophy','philosophy'),('cat-politics','politics'),('cat-career','career'),
  ('cat-environment','environment'),('cat-animals','animals'),('cat-nature','nature'),
  ('cat-events','events'),('cat-cars','cars'),('cat-space','space'),('cat-diy','diy'),
  ('cat-architecture','architecture'),('cat-languages','languages'),('cat-coding','coding'),
  ('cat-android','android'),('cat-ios','ios'),('cat-web-development','web development'),
  ('cat-ai','ai'),('cat-ml','ml'),('cat-data-science','data science'),
  ('cat-devops','devops'),('cat-security','security'),('cat-opensource','opensource')
ON CONFLICT (id) DO NOTHING;

-- Migrate legacy groups.category TEXT[] values into the map where they match (best-effort)
-- legacy column is TEXT[] after 20250526110936; older installs may still have VARCHAR. Guard it.
DO $$ BEGIN
  INSERT INTO group_category_map(group_id, category_id)
  SELECT g.id, c.id
  FROM groups g
  CROSS JOIN LATERAL unnest(COALESCE(g.category::text[], ARRAY[]::text[])) AS cat(name)
  JOIN group_categories c ON lower(c.name) = lower(cat.name)
  ON CONFLICT DO NOTHING;
EXCEPTION WHEN OTHERS THEN
  -- legacy category column has an incompatible type; skip best-effort migration
  NULL;
END $$;

-- 3. Membership with per-member read cursor (powers unread counts + badge)
CREATE TABLE IF NOT EXISTS group_members(
  group_id VARCHAR NOT NULL REFERENCES groups(id) ON DELETE CASCADE,
  user_name VARCHAR(50) NOT NULL REFERENCES profiles(user_name) ON DELETE CASCADE,
  role VARCHAR(20) NOT NULL DEFAULT 'member',
  last_read_at TIMESTAMP NOT NULL DEFAULT NOW(),
  last_read_msg_id VARCHAR,
  created_at TIMESTAMP NOT NULL DEFAULT NOW(),
  PRIMARY KEY (group_id, user_name)
);
CREATE INDEX IF NOT EXISTS idx_group_members_user ON group_members(user_name);
CREATE INDEX IF NOT EXISTS idx_group_members_group ON group_members(group_id);

-- Backfill: group owners become members (so old groups keep working)
INSERT INTO group_members(group_id, user_name, role)
SELECT id, user_name, 'owner' FROM groups
ON CONFLICT DO NOTHING;

-- 4. Topics: only one open per group (partial unique index)
CREATE TABLE IF NOT EXISTS group_topics(
  id VARCHAR PRIMARY KEY,
  group_id VARCHAR NOT NULL REFERENCES groups(id) ON DELETE CASCADE,
  title TEXT NOT NULL,
  is_open BOOLEAN NOT NULL DEFAULT true,
  created_by VARCHAR(50) NOT NULL,
  created_at TIMESTAMP NOT NULL DEFAULT NOW(),
  closed_at TIMESTAMP
);
CREATE UNIQUE INDEX IF NOT EXISTS one_open_topic ON group_topics(group_id) WHERE is_open;
CREATE INDEX IF NOT EXISTS idx_group_topics_group ON group_topics(group_id, created_at DESC);

-- 5. Messages: single feed per group
CREATE TABLE IF NOT EXISTS group_messages(
  id VARCHAR PRIMARY KEY,
  group_id VARCHAR NOT NULL REFERENCES groups(id) ON DELETE CASCADE,
  sender VARCHAR(50) NOT NULL REFERENCES profiles(user_name) ON DELETE CASCADE,
  text TEXT NOT NULL,
  image TEXT,
  topic_id VARCHAR REFERENCES group_topics(id) ON DELETE SET NULL,
  created_at TIMESTAMP NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_group_messages_feed ON group_messages(group_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_group_messages_id ON group_messages(group_id, id DESC);

-- 6. Join requests for private groups
CREATE TABLE IF NOT EXISTS group_join_requests(
  id VARCHAR PRIMARY KEY,
  group_id VARCHAR NOT NULL REFERENCES groups(id) ON DELETE CASCADE,
  user_name VARCHAR(50) NOT NULL REFERENCES profiles(user_name) ON DELETE CASCADE,
  status VARCHAR(20) NOT NULL DEFAULT 'pending',
  created_at TIMESTAMP NOT NULL DEFAULT NOW(),
  UNIQUE(group_id, user_name)
);
CREATE INDEX IF NOT EXISTS idx_group_join_requests_group ON group_join_requests(group_id, status);
