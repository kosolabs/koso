CREATE TABLE projects (
   id varchar(36) PRIMARY KEY NOT NULL,
   name varchar(36) NOT NULL
);

ALTER TABLE tasks
ADD project_id varchar(36);

INSERT INTO projects (id, name)
VALUES('koso-staging', 'Koso Staging');

UPDATE tasks
SET project_id='koso-staging'
WHERE project_id is NULL;

ALTER TABLE tasks
ALTER COLUMN project_id SET NOT NULL;
