use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sqlx::{types::Json, PgPool};

#[derive(Debug)]
pub(super) struct InstallationConfig {
    pub(super) installation_id: u64,
    pub(super) project_id: String,
}

#[derive(Clone)]
pub(super) struct ConfigStorage {
    pool: &'static PgPool,
}

#[derive(Deserialize, Serialize)]
struct ConfigData {
    project_id: String,
}

#[derive(sqlx::FromRow)]
struct ConfigRow {
    external_id: String,
    config: Json<ConfigData>,
}

impl ConfigStorage {
    pub(super) fn new(pool: &'static PgPool) -> Result<ConfigStorage> {
        Ok(ConfigStorage { pool })
    }

    pub(super) async fn get(&self, installation_id: u64) -> Result<InstallationConfig> {
        let (Json(config),): (Json<ConfigData>,) = sqlx::query_as(
            "
            SELECT config
            FROM plugin_configs
            WHERE plugin_id='github' and external_id=$1",
        )
        .bind(installation_id.to_string())
        .fetch_one(self.pool)
        .await
        .with_context(|| format!("Failed to get config for installation {installation_id}"))?;
        Ok(InstallationConfig {
            installation_id,
            project_id: config.project_id,
        })
    }

    pub(super) async fn list(&self) -> Result<Vec<InstallationConfig>> {
        let configs: Vec<ConfigRow> = sqlx::query_as(
            "
            SELECT
                external_id,
                config
            FROM plugin_configs
            WHERE plugin_id='github'",
        )
        .fetch_all(self.pool)
        .await
        .with_context(|| "Failed to list plugin configs")?;

        configs
            .into_iter()
            .map(|r| {
                Ok(InstallationConfig {
                    installation_id: r.external_id.parse::<u64>().with_context(|| {
                        format!("Failed to parse external_id {}", r.external_id)
                    })?,
                    project_id: r.config.0.project_id,
                })
            })
            .collect::<Result<Vec<_>>>()
    }
}
