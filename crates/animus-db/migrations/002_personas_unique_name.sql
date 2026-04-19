-- SQLite doesn't support ADD CONSTRAINT, so recreate the table.
-- Changes: UNIQUE(name), raw_card nullable.

PRAGMA foreign_keys = OFF;

CREATE TABLE personas_new (
    id              TEXT PRIMARY KEY,
    name            TEXT NOT NULL UNIQUE,
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
    raw_card        TEXT,
    created_at      INTEGER NOT NULL
);

INSERT INTO personas_new SELECT * FROM personas;
DROP TABLE personas;
ALTER TABLE personas_new RENAME TO personas;

PRAGMA foreign_keys = ON;
