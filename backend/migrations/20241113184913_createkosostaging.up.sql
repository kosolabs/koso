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
  (email, name, picture)
VALUES
  ('leonhard.kyle@gmail.com', 'Kyle Leonhard', 'https://lh3.googleusercontent.com/a/ACg8ocIIqNHG-bPON1NKXNOCiJR8fCS_ze3iIAsCvunJ4_kyhKJXFA=s96-c'),
  ('shadanan@gmail.com', 'Shad Sharma', 'https://lh3.googleusercontent.com/a/ACg8ocIRfl1MJrdKF_V8e46SQijmFzs1JoEaQLogCsOEIYC-T2Hk2xcPKw=s96-c')
ON CONFLICT DO NOTHING;