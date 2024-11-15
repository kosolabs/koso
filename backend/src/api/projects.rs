use crate::{
    api::{
        bad_request_error,
        collab::{storage, Collab},
        google::User,
        model::{
            CreateProject, Project, ProjectExport, ProjectUser, UpdateProjectUsers,
            UpdateProjectUsersResponse,
        },
        verify_project_access,
        yproxy::YDocProxy,
        ApiResult,
    },
    postgres::list_project_users,
};
use anyhow::Result;
use axum::{
    extract::Path,
    routing::{get, patch, post},
    Extension, Json, Router,
};
use base64::{prelude::BASE64_URL_SAFE_NO_PAD, Engine};
use sqlx::postgres::PgPool;
use uuid::Uuid;
use yrs::{ReadTxn as _, StateVector};

use super::verify_invited;

pub(super) fn router() -> Router {
    Router::new()
        .route("/", get(list_projects_handler))
        .route("/", post(create_project_handler))
        .route("/:project_id", get(get_project_handler))
        .route("/:project_id", patch(update_project_handler))
        .route("/:project_id/users", patch(update_project_users_handler))
        .route("/:project_id/users", get(list_project_users_handler))
        .route("/:project_id/updates", get(get_project_doc_updates_handler))
        .route("/:project_id/export", get(export_project))
}

#[tracing::instrument(skip(user, pool))]
async fn list_projects_handler(
    Extension(user): Extension<User>,
    Extension(pool): Extension<&'static PgPool>,
) -> ApiResult<Json<Vec<Project>>> {
    verify_invited(pool, &user).await?;

    let mut projects = list_projects(&user.email, pool).await?;
    projects.sort_by(|a, b| a.name.cmp(&b.name).then(a.project_id.cmp(&b.project_id)));
    Ok(Json(projects))
}

async fn list_projects(email: &String, pool: &PgPool) -> Result<Vec<Project>> {
    let projects: Vec<Project> = sqlx::query_as(
        "
        SELECT
          project_id,
          projects.name
        FROM project_permissions 
        JOIN projects USING(project_id)
        WHERE email = $1",
    )
    .bind(email)
    .fetch_all(pool)
    .await?;

    Ok(projects)
}

#[tracing::instrument(skip(user, pool))]
async fn create_project_handler(
    Extension(user): Extension<User>,
    Extension(pool): Extension<&'static PgPool>,
    Json(project): Json<CreateProject>,
) -> ApiResult<Json<Project>> {
    verify_invited(pool, &user).await?;

    let projects = list_projects(&user.email, pool).await?;
    const MAX_PROJECTS: usize = 20;
    if projects.len() >= MAX_PROJECTS {
        return Err(bad_request_error(
            "TOO_MANY_PROJECTS",
            &format!("Cannot create more than {} projects", MAX_PROJECTS),
        ));
    }
    validate_project_name(&project.name)?;

    let import_update = if let Some(import_data) = project.project_export {
        let ydoc = YDocProxy::new();
        let mut txn: yrs::TransactionMut<'_> = ydoc.transact_mut();
        for import_task in import_data.graph.values() {
            ydoc.set(&mut txn, import_task);
        }
        Some(txn.encode_state_as_update_v2(&StateVector::default()))
    } else {
        None
    };

    let project = Project {
        project_id: BASE64_URL_SAFE_NO_PAD.encode(Uuid::new_v4()),
        name: project.name,
    };

    let mut txn = pool.begin().await?;
    sqlx::query("INSERT INTO projects (project_id, name) VALUES ($1, $2)")
        .bind(&project.project_id)
        .bind(&project.name)
        .execute(&mut *txn)
        .await?;
    sqlx::query("INSERT INTO project_permissions (project_id, email) VALUES ($1, $2)")
        .bind(&project.project_id)
        .bind(&user.email)
        .execute(&mut *txn)
        .await?;
    if let Some(import_update) = import_update {
        sqlx::query("INSERT INTO yupdates (project_id, seq, update_v2) VALUES ($1, DEFAULT, $2)")
            .bind(&project.project_id)
            .bind(import_update)
            .execute(&mut *txn)
            .await?;
    }
    txn.commit().await?;

    tracing::debug!(
        "Created project '{}' with id '{}'",
        project.name,
        project.project_id
    );

    Ok(Json(project))
}

#[tracing::instrument(skip(user, pool))]
async fn list_project_users_handler(
    Extension(user): Extension<User>,
    Extension(pool): Extension<&'static PgPool>,
    Path(project_id): Path<String>,
) -> ApiResult<Json<Vec<ProjectUser>>> {
    verify_project_access(pool, user, &project_id).await?;
    let mut users = list_project_users(pool, &project_id).await?;
    users.sort_by(|a, b| a.name.cmp(&b.name).then(a.email.cmp(&b.email)));

    Ok(Json(users))
}

#[tracing::instrument(skip(user, pool))]
async fn get_project_handler(
    Extension(user): Extension<User>,
    Extension(pool): Extension<&'static PgPool>,
    Path(project_id): Path<String>,
) -> ApiResult<Json<Project>> {
    verify_project_access(pool, user, &project_id).await?;

    let project: Project = sqlx::query_as(
        "
        SELECT
          project_id,
          projects.name
        FROM projects
        WHERE project_id = $1",
    )
    .bind(project_id)
    .fetch_one(pool)
    .await?;

    Ok(Json(project))
}

#[tracing::instrument(skip(user, pool))]
async fn update_project_handler(
    Extension(user): Extension<User>,
    Extension(pool): Extension<&'static PgPool>,
    Path(project_id): Path<String>,
    Json(project): Json<Project>,
) -> ApiResult<Json<Project>> {
    verify_project_access(pool, user, &project_id).await?;

    if project_id != project.project_id {
        return Err(bad_request_error(
            "ID_MISMATCH",
            &format!(
                "Path project id ({project_id} is different than body project id {}",
                project.project_id
            ),
        ));
    }

    validate_project_name(&project.name)?;

    sqlx::query("UPDATE projects SET name=$2 WHERE project_id=$1")
        .bind(&project.project_id)
        .bind(&project.name)
        .execute(pool)
        .await?;
    Ok(Json(project))
}

#[tracing::instrument(skip(user, pool))]
async fn update_project_users_handler(
    Extension(user): Extension<User>,
    Extension(pool): Extension<&'static PgPool>,
    Path(project_id): Path<String>,
    Json(update): Json<UpdateProjectUsers>,
) -> ApiResult<Json<UpdateProjectUsersResponse>> {
    verify_project_access(pool, user, &project_id).await?;

    if project_id != update.project_id {
        return Err(bad_request_error(
            "ID_MISMATCH",
            &format!(
                "Path project id ({project_id} is different than body project id {}",
                update.project_id
            ),
        ));
    }
    if update.add_emails.is_empty() && update.remove_emails.is_empty() {
        return Ok(Json(UpdateProjectUsersResponse {}));
    }

    let add_emails = update
        .add_emails
        .into_iter()
        .map(|e| e.to_lowercase())
        .collect::<Vec<String>>();
    let remove_emails = update
        .remove_emails
        .into_iter()
        .map(|e| e.to_lowercase())
        .collect::<Vec<String>>();
    // Adds and removes may intersect. Assume that the add takes precedence below.

    let mut txn = pool.begin().await?;
    if !remove_emails.is_empty() {
        sqlx::query(
            "
            DELETE FROM project_permissions
            WHERE project_id=$1
            AND email in (SELECT * FROM unnest($2))",
        )
        .bind(&update.project_id)
        .bind(remove_emails)
        .execute(&mut *txn)
        .await?;
    }
    if !add_emails.is_empty() {
        sqlx::query(
            "
            INSERT INTO project_permissions (project_id, email) 
            SELECT $1, * FROM UNNEST($2)
            ON CONFLICT DO NOTHING",
        )
        .bind(&update.project_id)
        .bind(&add_emails)
        .execute(&mut *txn)
        .await?;

        sqlx::query(
            "
            UPDATE users
            SET invited=TRUE
            WHERE email in (SELECT * FROM unnest($2)) and NOT invited",
        )
        .bind(add_emails)
        .execute(&mut *txn)
        .await?;
    }

    txn.commit().await?;

    Ok(Json(UpdateProjectUsersResponse {}))
}

#[tracing::instrument(skip(user, pool))]
async fn get_project_doc_updates_handler(
    Extension(user): Extension<User>,
    Extension(pool): Extension<&'static PgPool>,
    Path(project_id): Path<String>,
) -> ApiResult<Json<Vec<String>>> {
    verify_project_access(pool, user, &project_id).await?;

    let updates = storage::load_updates(&project_id, pool)
        .await?
        .iter()
        .map(|u| u.to_string())
        .collect();
    Ok(Json(updates))
}

#[tracing::instrument(skip(user, pool, collab))]
async fn export_project(
    Extension(user): Extension<User>,
    Extension(pool): Extension<&'static PgPool>,
    Extension(collab): Extension<Collab>,
    Path(project_id): Path<String>,
) -> ApiResult<Json<ProjectExport>> {
    verify_project_access(pool, user, &project_id).await?;

    let graph = collab.get_graph(&project_id).await?;
    Ok(Json(ProjectExport { project_id, graph }))
}

fn validate_project_name(name: &str) -> ApiResult<()> {
    if name.is_empty() || name.chars().all(char::is_whitespace) {
        return Err(bad_request_error("EMPTY_NAME", "Project name is blank"));
    }
    const MAX_NAME_LEN: usize = 36;
    if name.len() > MAX_NAME_LEN {
        return Err(bad_request_error(
            "LONG_NAME",
            &format!(
                "Project name cannot be longer than {} characters ",
                MAX_NAME_LEN
            ),
        ));
    }
    Ok(())
}
