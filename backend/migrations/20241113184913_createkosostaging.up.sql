INSERT INTO projects
    (project_id, name)
VALUES
    ('koso-staging', 'Koso Staging');

INSERT INTO project_permissions
  (project_id, email)
VALUES
  ('koso-staging', 'leonhard.kyle@gmail.com'),
  ('koso-staging', 'shadanan@gmail.com');

INSERT INTO users
  (email, name)
VALUES
  ('leonhard.kyle@gmail.com', 'Kyle Leonhard'),
  ('shadanan@gmail.com', 'Shad Sharma')
ON CONFLICT DO NOTHING;