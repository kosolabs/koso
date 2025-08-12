pub mod app;
pub mod connect;
pub mod models;
pub mod sync;

#[cfg(test)]
mod tests;

use crate::{
    api::{
        collab::Collab,
        google::User,
        model::Task,
        yproxy::{YDocProxy, YTaskProxy},
    },
    plugins::{PluginSettings, config::ConfigStorage},
};
use anyhow::{Context, Result};
use axum::{Router, middleware};
use sqlx::PgPool;
use std::{cell::LazyCell, time::SystemTime};
use tokio::task::JoinHandle;
use yrs::TransactionMut;

use self::app::AppGoogleWorkspace;
use self::connect::ConnectHandler;
use self::sync::SyncService;

const PLUGIN_KIND: &Kind = &Kind::new("google_workspace", "Google Workspace");
const COMMENT_KIND: &Kind = &Kind::new_nested(PLUGIN_KIND, "google_comment", "Google Comment");
const REVIEWER_KIND: &Kind = &Kind::new_nested(PLUGIN_KIND, "google_reviewer", "Google Reviewer");

#[derive(Clone)]
pub(crate) struct Plugin {
    collab: Collab,
    config_storage: ConfigStorage,
    client: AppGoogleWorkspace,
    pool: &'static PgPool,
    settings: PluginSettings,
}

impl Plugin {
    pub(crate) async fn new(
        settings: PluginSettings,
        collab: Collab,
        pool: &'static PgPool,
    ) -> Result<Plugin> {
        PLUGIN_KIND.validate()?;
        COMMENT_KIND.validate()?;
        REVIEWER_KIND.validate()?;

        let client: AppGoogleWorkspace = AppGoogleWorkspace::new().await?;
        let config_storage = ConfigStorage::new(pool)?;

        Ok(Plugin {
            collab,
            client,
            config_storage,
            pool,
            settings,
        })
    }

    /// Start a background task that syncs Google Workspace documents periodically.
    /// Return a handle to the task, useful for aborting the task on shutdown.
    pub(crate) fn start_syncing(&self) -> JoinHandle<()> {
        if !self.settings.disable_polling {
            let sync_service = SyncService::new(
                self.collab.clone(),
                self.client.clone(),
                self.config_storage.clone(),
                self.pool,
            );
            tokio::spawn(sync_service.run())
        } else {
            tokio::spawn(async { tracing::debug!("Google Workspace sync disabled") })
        }
    }

    /// Returns a router that binds connection and sync endpoints.
    pub(crate) fn router(&self) -> Result<Router> {
        Ok(Router::new()
            .merge(
                ConnectHandler::new(self.pool, self.config_storage.clone(), self.client.clone())?
                    .router(),
            )
            .layer(middleware::from_fn(crate::api::google::authenticate))
            .merge(
                SyncService::new(
                    self.collab.clone(),
                    self.client.clone(),
                    self.config_storage.clone(),
                    self.pool,
                )
                .router()?,
            ))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct Kind {
    pub(crate) id: String,
    pub(crate) name: String,
}

impl Kind {
    pub(crate) const fn new(id: &'static str, name: &'static str) -> &'static Kind {
        &Kind {
            id: id.to_string(),
            name: name.to_string(),
        }
    }

    pub(crate) const fn new_nested(
        parent: &'static Kind,
        id: &'static str,
        name: &'static str,
    ) -> &'static Kind {
        &Kind {
            id: format!("{}_{}", parent.id, id),
            name: name.to_string(),
        }
    }

    pub(crate) fn validate(&self) -> Result<()> {
        if self.id.is_empty() {
            return Err(anyhow::anyhow!("Kind ID cannot be empty"));
        }
        if self.name.is_empty() {
            return Err(anyhow::anyhow!("Kind name cannot be empty"));
        }
        Ok(())
    }
}
