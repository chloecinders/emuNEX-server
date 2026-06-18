ALTER TABLE roms ADD COLUMN IF NOT EXISTS rom_files JSONB NOT NULL DEFAULT '[]'::jsonb;

UPDATE roms
SET rom_files = jsonb_build_array(
  jsonb_build_object(
    's3_path', rom_path,
    'filename', COALESCE(substring(rom_path from '[^/]+$'), 'rom.bin'),
    'is_entry', true
  )
)
WHERE rom_files = '[]'::jsonb AND rom_path IS NOT NULL AND rom_path != '';
