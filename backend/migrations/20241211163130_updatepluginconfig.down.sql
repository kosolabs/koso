CREATE TEMP TABLE plugin_configs_migration AS
SELECT * FROM plugin_configs;

DROP TABLE plugin_configs;

CREATE TABLE plugin_configs (
    plugin_id varchar(64) NOT NULL,
    external_id varchar(64) NOT NULL,
    config jsonb NOT NULL,
    PRIMARY KEY (plugin_id, external_id)
);

INSERT INTO plugin_configs
SELECT
    t.plugin_id,
    t.external_id,
    jsonb_build_object('project_id', t.project_id) AS config
FROM plugin_configs_migration AS t;
