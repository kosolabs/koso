CREATE TABLE oauth_clients (
    client_id varchar(64) NOT NULL,
    creation_time TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    client_metadata jsonb NOT NULL,
    PRIMARY KEY (client_id)
);
