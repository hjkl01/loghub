-- Add source location fields to logs
ALTER TABLE logs ADD COLUMN file_name VARCHAR(512);
ALTER TABLE logs ADD COLUMN function_name VARCHAR(256);
ALTER TABLE logs ADD COLUMN line_number INTEGER;

-- Index for file_name search
CREATE INDEX idx_logs_file_name ON logs(file_name);
