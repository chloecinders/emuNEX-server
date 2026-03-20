ALTER TABLE emulators ADD COLUMN consoles TEXT[] DEFAULT '{}';
UPDATE emulators SET consoles = ARRAY[console] WHERE console IS NOT NULL;

DROP INDEX IF EXISTS idx_emulators_console_platform;

ALTER TABLE emulators DROP COLUMN console;

CREATE INDEX idx_emulators_consoles ON emulators USING GIN (consoles);
