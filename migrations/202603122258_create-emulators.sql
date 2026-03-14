CREATE TABLE emulators (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    console VARCHAR(50) NOT NULL,
    platform VARCHAR(50) NOT NULL,
    run_command TEXT NOT NULL,
    binary_path TEXT NOT NULL UNIQUE,
    md5_hash CHAR(32),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_emulators_console_platform ON emulators(console, platform);
CREATE INDEX idx_emulators_name ON emulators(name);
