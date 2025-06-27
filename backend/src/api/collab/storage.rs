use super::{
    YDocProxy,
    txn_origin::{self, YOrigin},
};
use crate::api::model::ProjectId;
use anyhow::{Context as _, Result, anyhow};
use sqlx::{PgPool, Postgres};
use yrs::{
    Update,
    updates::{decoder::Decode, encoder::Encode},
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

#[tracing::instrument(skip(pool))]
pub(crate) async fn compact(pool: &PgPool, project_id: ProjectId) {
    if let Err(e) = _compact(pool, project_id).await {
        tracing::warn!("Failed to compact: {e:?}");
    }
}

async fn _compact(pool: &PgPool, project_id: ProjectId) -> Result<()> {
    tracing::debug!("Starting compaction");
    let mut txn = pool.begin().await?;

    let updates: Vec<(i32, Vec<u8>)> = sqlx::query_as(
        "
        SELECT seq, update_v2
        FROM yupdates
        WHERE project_id = $1
        ORDER BY seq ASC
        LIMIT 100",
    )
    .bind(&project_id)
    .fetch_all(&mut *txn)
    .await?;
    if updates.len() <= 1 {
        tracing::debug!("only {} updates exist, skipping compaction", updates.len());
        return Ok(());
    }

    let consumed_sequences = updates.iter().map(|(seq, _)| *seq).collect::<Vec<_>>();
    let Some(last_sequence) = consumed_sequences.iter().max() else {
        return Err(anyhow!("Could not get max sequence number"));
    };
    let merged_update = Update::merge_updates(
        updates
            .into_iter()
            .map(|(_, update)| Update::decode_v2(&update))
            .collect::<Result<Vec<_>, _>>()?,
    )
    .encode_v2();

    let deletes = sqlx::query(
        "
        DELETE FROM yupdates
        WHERE project_id = $1
        AND seq IN (SELECT unnest($2::integer[]))",
    )
    .bind(&project_id)
    .bind(&consumed_sequences)
    .execute(&mut *txn)
    .await?;
    if deletes.rows_affected() != consumed_sequences.len() as u64 {
        // This would only happen if multiple compactions and inserts interleave, which can happen with the
        // default postgres "read committed" isolation levels.
        // For example, after compaction A selects rows to compact, say 55, an update is inserted and compaction B sees 56 rows.
        // Compaction A would merge 1-55 and insert as 55 while compaction B would merge 1-56 and insert as 56.
        return Err(anyhow!(
            "Expected to delete {} yupdates, but actually deleted {}. Expected sequences: {consumed_sequences:?}",
            consumed_sequences.len(),
            deletes.rows_affected()
        ));
    }

    sqlx::query(
        "
        INSERT INTO yupdates (project_id, seq, update_v2)
        VALUES ($1, $2, $3)",
    )
    .bind(&project_id)
    .bind(last_sequence)
    .bind(merged_update)
    .execute(&mut *txn)
    .await?;

    txn.commit().await?;

    tracing::debug!("Compacted {} updates", consumed_sequences.len());
    Ok(())
}
