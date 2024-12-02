CREATE TABLE plugin_configs (
    plugin_id varchar(64) NOT NULL,
    external_id varchar(64) NOT NULL,
    config jsonb NOT NULL,
    PRIMARY KEY (plugin_id, external_id)
);
