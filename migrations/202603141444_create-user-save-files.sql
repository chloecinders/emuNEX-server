CREATE TABLE IF NOT EXISTS user_save_files (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL,
    rom_id INTEGER NOT NULL,
    version_id INTEGER NOT NULL,
    file_name TEXT NOT NULL,
    save_path TEXT NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_user_save_files_lookup ON user_save_files(user_id, rom_id, version_id);
