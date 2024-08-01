use crate::notify::ProjectId;
use sqlx::PgPool;
use std::error::Error;
use yrs::{
    updates::{decoder::Decode, encoder::Encode},
    Update,
};

#[tracing::instrument(skip(pool))]
pub async fn compact(pool: &PgPool, project_id: ProjectId) {
    if let Err(e) = _compact(pool, project_id).await {
        tracing::warn!("Failed to compact: {e}");
    }
}

async fn _compact(pool: &PgPool, project_id: ProjectId) -> Result<(), Box<dyn Error>> {
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
        return Err("Could not get max sequence number".into());
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
        return Err(format!("Expected to delete {} yupdates, but actually deleted {}. Expected sequences: {consumed_sequences:?}", consumed_sequences.len(), deletes.rows_affected()).into());
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
