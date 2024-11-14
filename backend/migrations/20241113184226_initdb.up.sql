CREATE TABLE IF NOT EXISTS projects (
    project_id varchar(36) NOT NULL,
    name varchar(36) NOT NULL,
    PRIMARY KEY (project_id)
);

CREATE TABLE IF NOT EXISTS users (
    email varchar(320) NOT NULL,
    name varchar(255) NOT NULL,
    picture varchar(2048) NOT NULL,
    PRIMARY KEY (email)
);

CREATE TABLE IF NOT EXISTS project_permissions (
    project_id varchar(36) NOT NULL,
    email varchar(320) NOT NULL,
    PRIMARY KEY (project_id, email)
);

CREATE TABLE IF NOT EXISTS yupdates (
    project_id varchar(36) NOT NULL,
    seq SERIAL NOT NULL,
    update_v2 bytea NOT NULL,
    PRIMARY KEY (project_id, seq)
);


