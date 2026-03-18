CREATE TABLE IF NOT EXISTS rom_reports (
    id TEXT PRIMARY KEY,
    rom_id TEXT NOT NULL REFERENCES roms(id) ON DELETE CASCADE,
    user_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    report_type TEXT NOT NULL,
    description TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'open',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_rom_reports_status ON rom_reports(status);
