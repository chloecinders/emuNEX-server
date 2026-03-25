CREATE TABLE search_sections (
    id BIGINT PRIMARY KEY,
    title TEXT NOT NULL,
    section_type TEXT NOT NULL,
    smart_filter TEXT,
    filter_value TEXT,
    order_index INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE search_section_roms (
    section_id BIGINT NOT NULL,
    rom_id TEXT NOT NULL,
    order_index INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (section_id, rom_id),
    FOREIGN KEY (section_id) REFERENCES search_sections(id) ON DELETE CASCADE,
    FOREIGN KEY (rom_id) REFERENCES roms(id) ON DELETE CASCADE
);

INSERT INTO search_sections (id, title, section_type, smart_filter, order_index)
VALUES
(1, 'Most Played', 'smart', 'most_played', 0),
(2, 'Recently Added', 'smart', 'recently_added', 1);
