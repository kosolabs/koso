use anyhow::{Context, Result};
use serde::{de::DeserializeOwned, Serialize};
use sqlx::{types::Json, PgPool};

#[derive(Clone)]
pub(super) struct ConfigStorage {
    pool: &'static PgPool,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Config<T> {
    pub(crate) plugin_id: String,
    pub(crate) external_id: String,
    /// Plugin specific configuration.
    pub(crate) config: T,
}

impl ConfigStorage {
    pub(super) fn new(pool: &'static PgPool) -> Result<ConfigStorage> {
        Ok(ConfigStorage { pool })
    }

    /// Get configuration for a specific plugin and scope.
    /// For example, the Github Plugin for a specific installation.
    pub(super) async fn get<T: 'static + Send + DeserializeOwned + Unpin>(
        &self,
        plugin_id: &str,
        external_id: &str,
    ) -> Result<Option<Config<T>>> {
        let Some((plugin_id, external_id, Json(config))): Option<(String, String, Json<T>)> =
            sqlx::query_as(
                "
            SELECT
                plugin_id,
                external_id,
                config
            FROM plugin_configs
            WHERE plugin_id=$1 and external_id=$2",
            )
            .bind(plugin_id)
            .bind(external_id)
            .fetch_optional(self.pool)
            .await
            .with_context(|| {
                format!("Failed to get plugin config for {plugin_id}:{external_id}")
            })?
        else {
            return Ok(None);
        };

        Ok(Some(Config {
            plugin_id,
            external_id,
            config,
        }))
    }

    /// Lists all configurations for the given plugin.
    pub(super) async fn list<T: 'static + Send + DeserializeOwned + Unpin>(
        &self,
        plugin_id: &str,
    ) -> Result<Vec<Config<T>>> {
        let configs: Vec<(String, String, Json<T>)> = sqlx::query_as(
            "
            SELECT
                plugin_id,
                external_id,
                config
            FROM plugin_configs
            WHERE plugin_id=$1",
        )
        .bind(plugin_id)
        .fetch_all(self.pool)
        .await
        .with_context(|| format!("Failed to list plugin configs for {plugin_id}"))?;

        Ok(configs
            .into_iter()
            .map(|(plugin_id, external_id, Json(config))| Config {
                plugin_id,
                external_id,
                config,
            })
            .collect())
    }

    pub(super) async fn insert_or_update<T: 'static + Send + Serialize + Unpin>(
        &self,
        config: &Config<T>,
    ) -> Result<()> {
        sqlx::query(
            "
        INSERT INTO plugin_configs (plugin_id, external_id, config)
        VALUES ($1, $2, $3)
        ON CONFLICT (plugin_id, external_id)
        DO UPDATE SET config = EXCLUDED.config;",
        )
        .bind(&config.plugin_id)
        .bind(&config.external_id)
        .bind(sqlx::types::Json(&config.config))
        .execute(self.pool)
        .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use serde::{Deserialize, Serialize};

    use super::*;

    #[derive(Debug, Deserialize, Serialize, PartialEq)]
    pub(super) struct SomePluginConfig {
        pub(super) project_id: String,
    }

    type SomeConfig = Config<SomePluginConfig>;

    #[test_log::test(sqlx::test)]
    async fn config_test(pool: PgPool) -> Result<()> {
        let pool = Box::leak(Box::new(pool.clone()));
        let storage = ConfigStorage { pool };

        storage
            .insert_or_update(&Config {
                plugin_id: "plugin_id_1".to_string(),
                external_id: "external_id_1".to_string(),
                config: SomePluginConfig {
                    project_id: "project_id_1".to_string(),
                },
            })
            .await?;

        let expected = SomeConfig {
            plugin_id: "plugin_id_1".to_string(),
            external_id: "external_id_1".to_string(),
            config: SomePluginConfig {
                project_id: "project_id_1".to_string(),
            },
        };
        let actual: SomeConfig = storage
            .get("plugin_id_1", "external_id_1")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(actual, expected);

        let actual: Vec<SomeConfig> = storage.list("plugin_id_1").await.unwrap();
        assert_eq!(actual, vec![expected]);

        let actual: Vec<SomeConfig> = storage.list("plugin_id_not_found").await.unwrap();
        assert_eq!(actual, vec![]);

        Ok(())
    }
}
