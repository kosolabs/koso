-- Add up migration script here
ALTER TABLE projects
ADD COLUMN deleted_on TIMESTAMP WITH TIME ZONE NULL DEFAULT NULL;
