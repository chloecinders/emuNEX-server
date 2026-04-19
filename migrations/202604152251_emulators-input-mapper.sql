ALTER TABLE emulators DROP COLUMN config_files;
ALTER TABLE emulators ADD COLUMN input_config_file TEXT;
ALTER TABLE emulators ADD COLUMN input_mapper TEXT;
