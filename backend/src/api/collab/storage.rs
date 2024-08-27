use crate::api::model::ProjectId;
use anyhow::{anyhow, Result};
use sqlx::PgPool;
use yrs::{updates::decoder::Decode as _, Doc, Transact as _, Update};

pub(super) async fn persist_update(
    project_id: &ProjectId,
    data: &Vec<u8>,
    pool: &PgPool,
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

pub(super) async fn load_doc(project_id: &ProjectId, pool: &PgPool) -> Result<(Doc, usize)> {
    let updates: Vec<(Vec<u8>,)> =
        sqlx::query_as("SELECT update_v2 FROM yupdates WHERE project_id=$1")
            .bind(project_id)
            .fetch_all(pool)
            .await?;
    let update_count = updates.len();

    let doc = Doc::new();
    {
        let mut txn = doc.transact_mut();
        for (update,) in updates {
            if let Err(e) = txn.apply_update(Update::decode_v2(&update)?) {
                return Err(anyhow!("Failed to apply loaded update: {e}"));
            }
        }
    }
    Result::Ok((doc, update_count))
}
