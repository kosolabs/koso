use crate::api::{google::User, model::ProjectId, verify_project_access};
use anyhow::{Context, Result};
use axum::{Extension, Json, extract::Path};
use axum_anyhow::{ApiResult, OptionExt, bad_request};
use base64::{Engine, prelude::BASE64_URL_SAFE_NO_PAD};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::{
    postgres::PgPool,
    types::chrono::{DateTime, Utc},
};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DedupeCandidate {
    pub(crate) dupe_id: String,
    pub(crate) project_id: ProjectId,
    pub(crate) task_1_id: String,
    pub(crate) task_2_id: String,
    pub(crate) similarity: Decimal,
    pub(crate) detected_at: DateTime<Utc>,
    pub(crate) resolution: Option<bool>,
    pub(crate) resolved_at: Option<DateTime<Utc>>,
    pub(crate) resolved_by: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CreateDupeCandidate {
    pub(crate) task_1_id: String,
    pub(crate) task_2_id: String,
    pub(crate) similarity: Decimal,
}

#[tracing::instrument(skip(user, pool))]
pub(crate) async fn list_dupes_handler(
    Extension(user): Extension<User>,
    Extension(pool): Extension<&'static PgPool>,
    Path(project_id): Path<String>,
) -> ApiResult<Json<Vec<DedupeCandidate>>> {
    verify_project_access(pool, &user, &project_id).await?;

    let dupes = list_dedupe_candidates(&project_id, pool).await?;
    Ok(Json(dupes))
}

#[tracing::instrument(skip(user, pool))]
pub(crate) async fn create_dupe_handler(
    Extension(user): Extension<User>,
    Extension(pool): Extension<&'static PgPool>,
    Path(project_id): Path<String>,
    Json(create_dupe): Json<CreateDupeCandidate>,
) -> ApiResult<Json<DedupeCandidate>> {
    verify_project_access(pool, &user, &project_id).await?;

    // Validate that task IDs are different
    if create_dupe.task_1_id == create_dupe.task_2_id {
        return Err(bad_request("SAME_TASK_IDS", "Task IDs must be different"));
    }

    // Validate similarity is between 0 and 1
    if create_dupe.similarity < Decimal::from(0) || create_dupe.similarity > Decimal::from(1) {
        return Err(bad_request(
            "INVALID_SIMILARITY",
            "Similarity must be between 0 and 1",
        ));
    }

    let dupe = create_dedupe_candidate(&project_id, &create_dupe, pool).await?;
    Ok(Json(dupe))
}

pub(crate) async fn list_dedupe_candidates(
    project_id: &ProjectId,
    pool: &PgPool,
) -> Result<Vec<DedupeCandidate>> {
    let mut candidates: Vec<DedupeCandidate> = sqlx::query_as(
        "
        SELECT
            dupe_id,
            project_id,
            task_1_id,
            task_2_id,
            similarity,
            detected_at,
            resolution,
            resolved_at,
            resolved_by
        FROM dedupe_candidates
        WHERE project_id = $1
        ",
    )
    .bind(project_id)
    .fetch_all(pool)
    .await
    .context("Failed to list dedupe candidates")?;

    // Sort by similarity DESC, detected_at DESC, task_1_id ASC, task_2_id ASC
    candidates.sort_by(|a, b| {
        b.similarity
            .cmp(&a.similarity)
            .then(b.detected_at.cmp(&a.detected_at))
            .then(a.task_1_id.cmp(&b.task_1_id))
            .then(a.task_2_id.cmp(&b.task_2_id))
    });

    Ok(candidates)
}

pub(crate) async fn create_dedupe_candidate(
    project_id: &ProjectId,
    create_dupe: &CreateDupeCandidate,
    pool: &PgPool,
) -> Result<DedupeCandidate> {
    let dupe_id = BASE64_URL_SAFE_NO_PAD.encode(Uuid::new_v4());

    let candidate: DedupeCandidate = sqlx::query_as(
        "
        INSERT INTO dedupe_candidates (
            dupe_id,
            project_id, 
            task_1_id, 
            task_2_id, 
            similarity, 
            detected_at
        ) VALUES ($1, $2, $3, $4, $5, NOW())
        ON CONFLICT (project_id, task_1_id, task_2_id) 
        DO UPDATE SET
            similarity = EXCLUDED.similarity,
            detected_at = NOW()
        RETURNING 
            dupe_id,
            project_id,
            task_1_id,
            task_2_id,
            similarity,
            detected_at,
            resolution,
            resolved_at,
            resolved_by
        ",
    )
    .bind(&dupe_id)
    .bind(project_id)
    .bind(&create_dupe.task_1_id)
    .bind(&create_dupe.task_2_id)
    .bind(create_dupe.similarity)
    .fetch_one(pool)
    .await
    .context("Failed to create dedupe candidate")?;

    Ok(candidate)
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ResolutionUpdate {
    pub resolution: Option<bool>,
}

pub(crate) async fn get_dupe_handler(
    Extension(user): Extension<User>,
    Extension(pool): Extension<&'static PgPool>,
    Path((project_id, dupe_id)): Path<(String, String)>,
) -> ApiResult<Json<DedupeCandidate>> {
    verify_project_access(pool, &user, &project_id).await?;

    let dupe = get_dedupe_candidate(&dupe_id, &project_id, pool).await?;
    Ok(Json(dupe))
}

pub(crate) async fn update_dupe_resolution_handler(
    Extension(user): Extension<User>,
    Extension(pool): Extension<&'static PgPool>,
    Path((project_id, dupe_id)): Path<(String, String)>,
    Json(resolution_update): Json<ResolutionUpdate>,
) -> ApiResult<Json<DedupeCandidate>> {
    verify_project_access(pool, &user, &project_id).await?;

    let updated_dupe = update_dedupe_resolution(
        &dupe_id,
        &project_id,
        resolution_update.resolution,
        &user.email,
        pool,
    )
    .await?;
    Ok(Json(updated_dupe))
}

pub(crate) async fn get_dedupe_candidate(
    dupe_id: &str,
    project_id: &ProjectId,
    pool: &PgPool,
) -> ApiResult<DedupeCandidate> {
    sqlx::query_as(
        "
        SELECT
            dupe_id,
            project_id,
            task_1_id,
            task_2_id,
            similarity,
            detected_at,
            resolution,
            resolved_at,
            resolved_by
        FROM dedupe_candidates
        WHERE dupe_id = $1 AND project_id = $2
        ",
    )
    .bind(dupe_id)
    .bind(project_id)
    .fetch_optional(pool)
    .await
    .context("Failed to get dedupe candidate")?
    .context_not_found("NOT_FOUND", "Dedupe candidate not found")
}

pub(crate) async fn update_dedupe_resolution(
    dupe_id: &str,
    project_id: &ProjectId,
    resolution: Option<bool>,
    resolved_by: &str,
    pool: &PgPool,
) -> ApiResult<DedupeCandidate> {
    sqlx::query_as(
        "
        UPDATE dedupe_candidates
        SET
            resolution = $3,
            resolved_at = CASE WHEN $3 IS NOT NULL THEN NOW() ELSE NULL END,
            resolved_by = CASE WHEN $3 IS NOT NULL THEN $4 ELSE NULL END
        WHERE dupe_id = $1 AND project_id = $2
        RETURNING 
            dupe_id,
            project_id,
            task_1_id,
            task_2_id,
            similarity,
            detected_at,
            resolution,
            resolved_at,
            resolved_by
        ",
    )
    .bind(dupe_id)
    .bind(project_id)
    .bind(resolution)
    .bind(resolved_by)
    .fetch_optional(pool)
    .await
    .context("Failed to update dedupe candidate resolution")?
    .context_not_found("NOT_FOUND", "Dedupe candidate not found")
}
