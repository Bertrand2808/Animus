CREATE TABLE IF NOT EXISTS personas (
    id              TEXT PRIMARY KEY,
    name            TEXT NOT NULL,
    description     TEXT NOT NULL DEFAULT '',
    personality     TEXT NOT NULL DEFAULT '',
    scenario        TEXT NOT NULL DEFAULT '',
    first_message   TEXT NOT NULL DEFAULT '',
    message_example TEXT NOT NULL DEFAULT '',
    avatar_url      TEXT,
    background_url  TEXT,
    content_rating  TEXT NOT NULL DEFAULT 'pg'
                        CHECK(content_rating IN ('pg','mature','nsfw')),
    model           TEXT,
    raw_card        TEXT NOT NULL,
    created_at      INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS conversations (
    id          TEXT PRIMARY KEY,
    persona_id  TEXT NOT NULL REFERENCES personas(id) ON DELETE CASCADE,
    created_at  INTEGER NOT NULL,
    updated_at  INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS messages (
    id              TEXT PRIMARY KEY,
    conversation_id TEXT NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
    role            TEXT NOT NULL CHECK(role IN ('user','assistant','system')),
    content         TEXT NOT NULL,
    token_count     INTEGER,
    created_at      INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_messages_conversation
    ON messages(conversation_id, created_at);

CREATE TABLE IF NOT EXISTS summaries (
    id                   TEXT PRIMARY KEY,
    conversation_id      TEXT NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
    content              TEXT NOT NULL,
    message_range_start  TEXT NOT NULL,
    message_range_end    TEXT NOT NULL,
    created_at           INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_summaries_conversation
    ON summaries(conversation_id, created_at);
