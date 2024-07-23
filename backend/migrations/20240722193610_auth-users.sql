CREATE TABLE users (
   email varchar(320) PRIMARY KEY NOT NULL,
   name varchar(255) NOT NULL,
   picture varchar(2048)
);

ALTER TABLE tasks
ADD assignee varchar(320),
ADD reporter varchar(320);

UPDATE tasks
SET reporter='shadanan@gmail.com'
WHERE reporter is NULL;

ALTER TABLE tasks
ALTER COLUMN reporter SET NOT NULL;
