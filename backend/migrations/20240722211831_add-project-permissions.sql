CREATE TABLE project_permissions (
  project_id varchar(36) NOT NULL,
  email varchar(320) NOT NULL,
  PRIMARY KEY (project_id, email)
);

INSERT INTO project_permissions
  (project_id, email)
VALUES
  ('koso-staging', 'leonhard.kyle@gmail.com'),
  ('koso-staging', 'shadanan@gmail.com');