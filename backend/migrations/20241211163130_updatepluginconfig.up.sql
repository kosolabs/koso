CREATE TEMP TABLE plugin_configs_migration AS
SELECT * FROM plugin_configs;

DROP TABLE plugin_configs;

CREATE TABLE plugin_configs (
    project_id varchar(36) NOT NULL,
    plugin_id varchar(64) NOT NULL,
    external_id varchar(64) NOT NULL,
    settings jsonb NOT NULL,
    PRIMARY KEY (project_id, plugin_id, external_id)
);

INSERT INTO plugin_configs
SELECT
    t.config ->> 'project_id' AS project_id,
    t.plugin_id AS plugin_id,
    t.external_id AS external_id,
    '{}'::jsonb AS settings
FROM plugin_configs_migration as t;
