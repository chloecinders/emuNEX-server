ALTER TABLE users ALTER COLUMN password DROP NOT NULL;

CREATE TABLE IF NOT EXISTS discord_connections (
    user_id        BIGINT PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    discord_id     VARCHAR(32) NOT NULL UNIQUE,
    access_token   TEXT        NOT NULL,
    refresh_token  TEXT        NOT NULL,
    expires_at     TIMESTAMPTZ NOT NULL,
    linked_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_discord_connections_discord_id ON discord_connections(discord_id);
