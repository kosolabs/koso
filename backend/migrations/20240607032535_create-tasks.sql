-- Add migration script here
CREATE TABLE tasks (
   id varchar(36) PRIMARY KEY NOT NULL,
   name varchar(200) NOT NULL
);