-- Add migration script here
ALTER TABLE tasks
ADD children varchar(36)[] NOT NULL DEFAULT array[]::varchar[];