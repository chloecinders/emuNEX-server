DROP TABLE IF EXISTS shelf_roms CASCADE;
DROP TABLE IF EXISTS library_shelves CASCADE;
DROP TABLE IF EXISTS user_save_files CASCADE;
DROP TABLE IF EXISTS user_roms CASCADE;
DROP TABLE IF EXISTS user_tokens CASCADE;
DROP TABLE IF EXISTS emulators CASCADE;
DROP TABLE IF EXISTS roms CASCADE;
DROP TABLE IF EXISTS consoles CASCADE;
DROP TABLE IF EXISTS users CASCADE;
DROP TYPE IF EXISTS user_role CASCADE;

CREATE TYPE user_role AS ENUM ('admin', 'moderator', 'user');

CREATE TABLE users (
    id          BIGINT PRIMARY KEY,
    username    VARCHAR(50) UNIQUE NOT NULL,
    password    TEXT NOT NULL,
    role        user_role NOT NULL DEFAULT 'user',
    created_at  TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at  TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX idx_users_username ON users(username);

CREATE TABLE user_tokens (
    id          BIGINT PRIMARY KEY,
    user_id     BIGINT NOT NULL,
    token       TEXT UNIQUE NOT NULL,
    expires_at  TIMESTAMPTZ NOT NULL,
    created_at  TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    last_used_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT fk_user FOREIGN KEY(user_id) REFERENCES users(id) ON DELETE CASCADE
);
CREATE INDEX idx_user_tokens_user_id ON user_tokens(user_id);
CREATE INDEX idx_user_tokens_lookup ON user_tokens(token);

CREATE TABLE consoles (
    id         BIGINT PRIMARY KEY,
    name       TEXT UNIQUE NOT NULL,
    card_color TEXT
);

CREATE TABLE roms (
    id               TEXT PRIMARY KEY,
    title            VARCHAR(255) NOT NULL,
    console          VARCHAR(50) NOT NULL,
    region           VARCHAR(10),
    category         VARCHAR(255) NOT NULL,
    serial           TEXT,
    rom_path         TEXT NOT NULL UNIQUE,
    image_path       TEXT NOT NULL UNIQUE,
    file_extension   VARCHAR(10),
    file_size_bytes  BIGINT,
    md5_hash         CHAR(32),
    release_year     INTEGER,
    is_favorite      BOOLEAN DEFAULT FALSE,
    play_count       INTEGER DEFAULT 0,
    last_played      TIMESTAMPTZ,
    created_at       TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX idx_roms_console_title ON roms(console, title);
CREATE INDEX idx_roms_md5 ON roms(md5_hash);
CREATE INDEX idx_roms_serial ON roms(serial);

CREATE TABLE emulators (
    id            BIGINT PRIMARY KEY,
    name          VARCHAR(255) NOT NULL,
    console       VARCHAR(50) NOT NULL,
    platform      VARCHAR(50) NOT NULL,
    run_command   TEXT NOT NULL,
    binary_path   TEXT NOT NULL UNIQUE,
    md5_hash      CHAR(32),
    config_files  TEXT[] NOT NULL DEFAULT '{}',
    zipped        BOOLEAN NOT NULL DEFAULT FALSE,
    created_at    TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX idx_emulators_console_platform ON emulators(console, platform);
CREATE INDEX idx_emulators_name ON emulators(name);

CREATE TABLE user_roms (
    user_id     BIGINT NOT NULL,
    rom_id      TEXT NOT NULL,
    play_count  INT DEFAULT 0 NOT NULL,
    last_played TIMESTAMPTZ,
    added_at    TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (user_id, rom_id),
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (rom_id) REFERENCES roms(id) ON DELETE CASCADE
);

CREATE TABLE user_save_files (
    id          BIGINT PRIMARY KEY,
    user_id     BIGINT NOT NULL,
    rom_id      TEXT NOT NULL,
    version_id  BIGINT NOT NULL,
    file_name   TEXT NOT NULL,
    save_path   TEXT NOT NULL,
    created_at  TIMESTAMPTZ DEFAULT NOW()
);
CREATE INDEX idx_user_save_files_lookup ON user_save_files(user_id, rom_id, version_id);

CREATE TABLE library_shelves (
    id          BIGINT PRIMARY KEY,
    user_id     BIGINT NOT NULL,
    name        VARCHAR(255) NOT NULL,
    sort_order  INT DEFAULT 0 NOT NULL,
    created_at  TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    UNIQUE (user_id, name)
);
CREATE INDEX idx_library_shelves_user_order ON library_shelves(user_id, sort_order);

CREATE TABLE shelf_roms (
    shelf_id    BIGINT NOT NULL,
    rom_id      TEXT NOT NULL,
    sort_order  INT DEFAULT 0 NOT NULL,
    added_at    TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (shelf_id, rom_id),
    FOREIGN KEY (shelf_id) REFERENCES library_shelves(id) ON DELETE CASCADE,
    FOREIGN KEY (rom_id) REFERENCES roms(id) ON DELETE CASCADE
);
CREATE INDEX idx_shelf_roms_shelf_order ON shelf_roms(shelf_id, sort_order);
