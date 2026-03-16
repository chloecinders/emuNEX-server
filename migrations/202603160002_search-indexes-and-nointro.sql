CREATE EXTENSION IF NOT EXISTS pg_trgm;

CREATE INDEX idx_roms_title_trgm ON roms USING GIN (title gin_trgm_ops);
CREATE INDEX idx_roms_serial_trgm ON roms USING GIN (serial gin_trgm_ops);

CREATE TABLE nointro_games (
    id          TEXT PRIMARY KEY,
    dat_name    TEXT NOT NULL,
    name        TEXT NOT NULL,
    serial      TEXT,
    md5         CHAR(32),
    crc         CHAR(8),
    sha1        CHAR(40),
    sha256      CHAR(64),
    size        BIGINT,
    clone_of    TEXT,
    status      TEXT,
    created_at  TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_nointro_dat ON nointro_games(dat_name);
CREATE INDEX idx_nointro_serial ON nointro_games(serial);
CREATE INDEX idx_nointro_md5 ON nointro_games(md5);
CREATE INDEX idx_nointro_name_trgm ON nointro_games USING GIN (name gin_trgm_ops);
