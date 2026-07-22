-- Drop system column and its index
DROP INDEX IF EXISTS idx_logs_system_time;
ALTER TABLE logs DROP COLUMN system;
