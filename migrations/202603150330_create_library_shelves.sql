CREATE TABLE library_shelves (
    id SERIAL PRIMARY KEY,
    user_id INT NOT NULL,
    name VARCHAR(255) NOT NULL,
    sort_order INT DEFAULT 0 NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    UNIQUE (user_id, name)
);

CREATE TABLE shelf_roms (
    shelf_id INT NOT NULL,
    rom_id INT NOT NULL,
    sort_order INT DEFAULT 0 NOT NULL,
    added_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (shelf_id, rom_id),
    FOREIGN KEY (shelf_id) REFERENCES library_shelves(id) ON DELETE CASCADE,
    FOREIGN KEY (rom_id) REFERENCES roms(id) ON DELETE CASCADE
);

CREATE INDEX idx_library_shelves_user_order ON library_shelves(user_id, sort_order);
CREATE INDEX idx_shelf_roms_shelf_order ON shelf_roms(shelf_id, sort_order);
