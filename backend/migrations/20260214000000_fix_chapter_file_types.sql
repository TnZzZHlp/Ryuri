-- Fix invalid file_type in chapters (which contain full paths)
-- Extract extension from file_path where file_type is invalid (contains '/')
UPDATE chapters
SET file_type = LOWER(
    CASE
        WHEN INSTR(file_path, '.') > 0
        THEN SUBSTR(file_path, LENGTH(file_path) - LENGTH(REPLACE(file_path, '.', '')) + 1)
        ELSE '' -- Fallback if no extension found
    END
)
WHERE file_type LIKE '%/%' OR file_type LIKE '%\%';
