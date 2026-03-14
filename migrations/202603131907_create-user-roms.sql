CREATE TABLE user_roms (
    user_id INT NOT NULL,
    rom_id INT NOT NULL,
    play_count INT DEFAULT 0 NOT NULL,
    last_played TIMESTAMP WITH TIME ZONE,
    added_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (user_id, rom_id),
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (rom_id) REFERENCES roms(id) ON DELETE CASCADE
);
