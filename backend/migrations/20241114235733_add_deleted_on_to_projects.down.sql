-- Add down migration script here
ALTER TABLE projects
DROP COLUMN deleted_on CASCADE;
