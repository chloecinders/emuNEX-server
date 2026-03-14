CREATE TABLE consoles (
    id SERIAL PRIMARY KEY,
    name TEXT UNIQUE NOT NULL,
    card_color TEXT
);
