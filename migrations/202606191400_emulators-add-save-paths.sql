ALTER TABLE emulators ADD COLUMN save_paths JSONB DEFAULT '[]'::jsonb NOT NULL;

UPDATE emulators
SET save_paths = jsonb_build_array(save_path)
WHERE save_path IS NOT NULL AND save_path != '';

ALTER TABLE emulators DROP COLUMN save_path;
