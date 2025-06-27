use crate::api::model::ProjectId;
use anyhow::{Context as _, Result};
use sqlx::{PgPool, Postgres};
use yrs::{Update, updates::decoder::Decode as _};

use super::{
    YDocProxy,
    txn_origin::{self, YOrigin},
};

pub(in crate::api) async fn persist_update<'a, E: sqlx::Executor<'a, Database = Postgres>>(
    project_id: &ProjectId,
    data: &Vec<u8>,
    pool: E,
) -> Result<()> {
    sqlx::query(
        "
            INSERT INTO yupdates (project_id, seq, update_v2)
            VALUES ($1, DEFAULT, $2)",
    )
    .bind(project_id)
    .bind(data)
    .execute(pool)
    .await?;
    Ok(())
}

pub(super) async fn load_doc(project_id: &ProjectId, pool: &PgPool) -> Result<(YDocProxy, usize)> {
    let updates = load_raw_updates(project_id, pool).await?;
    let update_count = updates.len();

    let ydoc = YDocProxy::new();
    {
        let mut txn = ydoc.transact_mut_with(
            YOrigin {
                who: "load_doc".to_string(),
                id: project_id.to_string(),
                actor: txn_origin::Actor::Server,
            }
            .as_origin()?,
        );
        for (update,) in updates {
            txn.apply_update(Update::decode_v2(&update)?)
                .context("Failed to apply loaded update")?
        }
    }
    Result::Ok((ydoc, update_count))
}

pub async fn load_updates(project_id: &ProjectId, pool: &PgPool) -> Result<Vec<Update>> {
    let updates = load_raw_updates(project_id, pool).await?;
    let updates = updates
        .into_iter()
        .map(|u| Update::decode_v2(&u.0))
        .collect::<Result<Vec<_>, yrs::encoding::read::Error>>()?;

    Result::Ok(updates)
}

async fn load_raw_updates(project_id: &ProjectId, pool: &PgPool) -> Result<Vec<(Vec<u8>,)>> {
    let updates: Vec<(Vec<u8>,)> =
        sqlx::query_as("SELECT update_v2 FROM yupdates WHERE project_id=$1")
            .bind(project_id)
            .fetch_all(pool)
            .await?;
    Ok(updates)
}
