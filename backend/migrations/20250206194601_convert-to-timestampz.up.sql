ALTER TABLE projects ALTER deleted_on TYPE timestamptz;
ALTER TABLE users ALTER creation_time TYPE timestamptz;
ALTER TABLE users ALTER login_time TYPE timestamptz;
