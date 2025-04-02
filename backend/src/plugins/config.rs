use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, types::Json};

#[derive(Clone)]
pub(super) struct ConfigStorage {
    pool: &'static PgPool,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Config {
    pub(crate) project_id: String,
    pub(crate) plugin_id: String,
    pub(crate) external_id: String,
    /// Plugin specific configuration.
    pub(crate) settings: Settings,
}

#[derive(Serialize, Clone, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase", tag = "type")]
pub(crate) enum Settings {
    Github(GithubSettings),
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub(crate) struct GithubSettings {}

type ConfigRow = (String, String, String, Json<Settings>);

impl ConfigStorage {
    pub(super) fn new(pool: &'static PgPool) -> Result<ConfigStorage> {
        Ok(ConfigStorage { pool })
    }

    /// Lists all configurations for the given plugin.
    pub(super) async fn list_for_external_id(
        &self,
        plugin_id: &str,
        external_id: &str,
    ) -> Result<Vec<Config>> {
        let configs: Vec<ConfigRow> = sqlx::query_as(
            "
            SELECT
                project_id,
                plugin_id,
                external_id,
                settings
            FROM plugin_configs
            JOIN projects USING(project_id)
            WHERE plugin_id=$1 and external_id=$2 AND deleted_on IS NULL",
        )
        .bind(plugin_id)
        .bind(external_id)
        .fetch_all(self.pool)
        .await
        .with_context(|| format!("Failed to list plugin configs for {plugin_id}:{external_id}"))?;

        Ok(rows_to_configs(configs))
    }

    /// Lists all configurations for the given plugin.
    pub(super) async fn list_for_plugin(&self, plugin_id: &str) -> Result<Vec<Config>> {
        let configs: Vec<ConfigRow> = sqlx::query_as(
            "
            SELECT
                project_id,
                plugin_id,
                external_id,
                settings
            FROM plugin_configs
            JOIN projects USING(project_id)
            WHERE plugin_id=$1 AND deleted_on IS NULL",
        )
        .bind(plugin_id)
        .fetch_all(self.pool)
        .await
        .with_context(|| format!("Failed to list plugin configs for {plugin_id}"))?;

        Ok(rows_to_configs(configs))
    }

    pub(super) async fn insert_or_update(&self, config: &Config) -> Result<()> {
        sqlx::query(
            "
        INSERT INTO plugin_configs (project_id, plugin_id, external_id, settings)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (project_id, plugin_id, external_id)
        DO UPDATE SET settings = EXCLUDED.settings;",
        )
        .bind(&config.project_id)
        .bind(&config.plugin_id)
        .bind(&config.external_id)
        .bind(sqlx::types::Json(&config.settings))
        .execute(self.pool)
        .await?;
        Ok(())
    }
}

fn rows_to_configs(configs: Vec<ConfigRow>) -> Vec<Config> {
    configs
        .into_iter()
        .map(
            |(project_id, plugin_id, external_id, Json(settings))| Config {
                project_id,
                plugin_id,
                external_id,
                settings,
            },
        )
        .collect()
}

#[cfg(test)]
mod tests {
    use serde::{Deserialize, Serialize};

    use super::*;

    #[derive(Debug, Deserialize, Serialize, PartialEq)]
    pub(super) struct SomePluginSettings {
        pub(super) other_thing: String,
    }

    #[test_log::test(sqlx::test)]
    async fn config_test(pool: PgPool) -> Result<()> {
        let pool = Box::leak(Box::new(pool.clone()));
        let storage = ConfigStorage { pool };

        sqlx::query("INSERT INTO projects (project_id, name) VALUES ($1, $2)")
            .bind("project_id_1")
            .bind("config_test")
            .execute(&*pool)
            .await?;
        storage
            .insert_or_update(&Config {
                project_id: "project_id_1".to_string(),
                plugin_id: "plugin_id_1".to_string(),
                external_id: "external_id_1".to_string(),
                settings: Settings::Github(GithubSettings {}),
            })
            .await?;

        let expected = vec![Config {
            project_id: "project_id_1".to_string(),
            plugin_id: "plugin_id_1".to_string(),
            external_id: "external_id_1".to_string(),
            settings: Settings::Github(GithubSettings {}),
        }];

        let actual: Vec<Config> = storage.list_for_plugin("plugin_id_1").await.unwrap();
        assert_eq!(actual, expected);

        let actual: Vec<Config> = storage
            .list_for_external_id("plugin_id_1", "external_id_1")
            .await
            .unwrap();
        assert_eq!(actual, expected);

        let actual: Vec<Config> = storage
            .list_for_plugin("plugin_id_not_found")
            .await
            .unwrap();
        assert_eq!(actual, vec![]);

        Ok(())
    }

    #[test_log::test(sqlx::test)]
    async fn list_excludes_deleted_projects(pool: PgPool) -> Result<()> {
        let pool = Box::leak(Box::new(pool.clone()));
        let storage = ConfigStorage { pool };

        storage
            .insert_or_update(&Config {
                project_id: "project_id_1".to_string(),
                plugin_id: "plugin_id_1".to_string(),
                external_id: "external_id_1".to_string(),
                settings: Settings::Github(GithubSettings {}),
            })
            .await?;

        let actual: Vec<Config> = storage.list_for_plugin("plugin_id_1").await.unwrap();
        assert_eq!(actual, vec![]);

        let actual: Vec<Config> = storage
            .list_for_external_id("plugin_id_1", "external_id_1")
            .await
            .unwrap();
        assert_eq!(actual, vec![]);

        Ok(())
    }
}
