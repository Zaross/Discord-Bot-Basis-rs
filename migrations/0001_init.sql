CREATE TABLE IF NOT EXISTS guild_settings (
    guild_id    TEXT        NOT NULL PRIMARY KEY,
    prefix      TEXT        NOT NULL DEFAULT '/',
    language    TEXT        NOT NULL DEFAULT 'en',
    created_at  TIMESTAMP   NOT NULL DEFAULT CURRENT_TIMESTAMP
);