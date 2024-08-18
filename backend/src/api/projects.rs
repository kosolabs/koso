use crate::{
    api::{
        bad_request_error,
        collab::Collab,
        google::User,
        model::{CreateProject, Project, ProjectUser, UpdateProjectPermissions},
        verify_access, ApiResult,
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

pub(super) fn projects_router() -> Router {
    Router::new()
        .route("/", get(list_projects_handler))
        .route("/", post(create_project_handler))
        .route("/:project_id", patch(update_project_handler))
        .route(
            "/:project_id/permissions",
            patch(update_project_permissions_handler),
        )
        .route("/:project_id/users", get(list_project_users_handler))
        .route("/:project_id/doc", get(get_project_doc_handler))
}

#[tracing::instrument(skip(user, pool))]
async fn list_projects_handler(
    Extension(user): Extension<User>,
    Extension(pool): Extension<&'static PgPool>,
) -> ApiResult<Json<Vec<Project>>> {
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
    let projects = list_projects(&user.email, pool).await?;
    if projects.len() >= 5 {
        return Err(bad_request_error(&format!(
            "User has more than 5 projects ({})",
            projects.len()
        )));
    }

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
    verify_access(pool, user, &project_id).await?;
    let mut users = list_project_users(pool, &project_id).await?;
    users.sort_by(|a, b| a.name.cmp(&b.name).then(a.email.cmp(&b.email)));

    Ok(Json(users))
}

#[tracing::instrument(skip(user, pool))]
async fn update_project_handler(
    Extension(user): Extension<User>,
    Extension(pool): Extension<&'static PgPool>,
    Path(project_id): Path<String>,
    Json(project): Json<Project>,
) -> ApiResult<Json<Project>> {
    verify_access(pool, user, &project_id).await?;

    if project_id != project.project_id {
        return Err(bad_request_error(&format!(
            "Path project id ({project_id} is different than body project id {}",
            project.project_id
        )));
    }

    if project.name.is_empty() {
        return Err(bad_request_error("Project name is empty"));
    }

    sqlx::query("UPDATE projects SET name=$2 WHERE project_id=$1")
        .bind(&project.project_id)
        .bind(&project.name)
        .execute(pool)
        .await?;
    Ok(Json(project))
}

#[tracing::instrument(skip(user, pool))]
async fn update_project_permissions_handler(
    Extension(user): Extension<User>,
    Extension(pool): Extension<&'static PgPool>,
    Path(project_id): Path<String>,
    Json(update): Json<UpdateProjectPermissions>,
) -> ApiResult<()> {
    verify_access(pool, user, &project_id).await?;

    if project_id != update.project_id {
        return Err(bad_request_error(&format!(
            "Path project id ({project_id} is different than body project id {}",
            update.project_id
        )));
    }
    if update.add_emails.is_empty() && update.remove_emails.is_empty() {
        return Ok(());
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
        .bind(add_emails)
        .execute(&mut *txn)
        .await?;
    }

    txn.commit().await?;

    Ok(())
}

#[tracing::instrument(skip(user, pool, collab))]
async fn get_project_doc_handler(
    Extension(user): Extension<User>,
    Extension(pool): Extension<&'static PgPool>,
    Extension(collab): Extension<Collab>,
    Path(project_id): Path<String>,
) -> ApiResult<Json<yrs::Any>> {
    verify_access(pool, user, &project_id).await?;

    Ok(Json(collab.get_doc(&project_id).await?))
}
