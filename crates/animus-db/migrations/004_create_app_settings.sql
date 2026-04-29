CREATE TABLE IF NOT EXISTS app_settings (
    id            INTEGER PRIMARY KEY CHECK (id = 1),
    user_name     TEXT NOT NULL DEFAULT 'User',
    default_model TEXT NOT NULL DEFAULT 'gemma4'
);
