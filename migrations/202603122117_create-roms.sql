CREATE TABLE roms (
    id SERIAL PRIMARY KEY,

    title VARCHAR(255) NOT NULL,
    console VARCHAR(50) NOT NULL,
    region VARCHAR(10),
    category VARCHAR(255) NOT NULL,

    rom_path TEXT NOT NULL UNIQUE,
    image_path TEXT NOT NULL UNIQUE,

    file_extension VARCHAR(10),
    file_size_bytes BIGINT,
    md5_hash CHAR(32),

    release_year INTEGER,
    is_favorite BOOLEAN DEFAULT FALSE,
    play_count INTEGER DEFAULT 0,
    last_played TIMESTAMP WITH TIME ZONE,

    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_roms_console_title ON roms(console, title);
CREATE INDEX idx_roms_md5 ON roms(md5_hash);
