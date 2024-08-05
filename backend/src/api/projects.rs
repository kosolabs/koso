use crate::{
    api::{
        bad_request_error,
        google::User,
        model::{Project, ProjectPermission, ProjectUser},
        notify::Notifier,
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

pub fn projects_router() -> Router {
    Router::new()
        .route("/", get(list_projects_handler))
        .route("/", post(create_project_handler))
        .route("/:project_id", patch(update_project_handler))
        .route(
            "/:project_id/permissions",
            post(add_project_permission_handler),
        )
        .route("/:project_id/users", get(list_project_users_handler))
        .route("/:project_id/doc", get(get_project_doc_handler))
}

#[tracing::instrument(skip(user, pool))]
async fn list_projects_handler(
    Extension(user): Extension<User>,
    Extension(pool): Extension<&'static PgPool>,
) -> ApiResult<Json<Vec<Project>>> {
    let projects = list_projects(&user.email, pool).await?;
    Ok(Json(projects))
}

async fn list_projects(email: &String, pool: &PgPool) -> Result<Vec<Project>> {
    let projects: Vec<Project> = sqlx::query_as(
        "
        SELECT
          project_permissions.project_id,
          projects.name
        FROM project_permissions 
        JOIN projects ON (project_permissions.project_id = projects.id)
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
    Json(mut project): Json<Project>,
) -> ApiResult<Json<Project>> {
    let projects = list_projects(&user.email, pool).await?;
    if projects.len() >= 5 {
        return Err(bad_request_error(&format!(
            "User has more than 5 projects ({})",
            projects.len()
        )));
    }

    project.project_id = BASE64_URL_SAFE_NO_PAD.encode(Uuid::new_v4());

    let mut txn = pool.begin().await?;
    sqlx::query("INSERT INTO projects (id, name) VALUES ($1, $2)")
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
    let users = list_project_users(pool, &project_id).await?;
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

    sqlx::query("UPDATE projects SET name=$2 WHERE id=$1")
        .bind(&project.project_id)
        .bind(&project.name)
        .execute(pool)
        .await?;
    Ok(Json(project))
}

#[tracing::instrument(skip(user, pool))]
async fn add_project_permission_handler(
    Extension(user): Extension<User>,
    Extension(pool): Extension<&'static PgPool>,
    Path(project_id): Path<String>,
    Json(permission): Json<ProjectPermission>,
) -> ApiResult<()> {
    verify_access(pool, user, &project_id).await?;

    if project_id != permission.project_id {
        return Err(bad_request_error(&format!(
            "Path project id ({project_id} is different than body project id {}",
            permission.project_id
        )));
    }

    if permission.email.is_empty() {
        return Err(bad_request_error("Permission email is empty"));
    }

    sqlx::query("INSERT INTO project_permissions (project_id, email) VALUES ($1, $2) ON CONFLICT DO NOTHING")
            .bind(&permission.project_id)
            .bind(&permission.email)
            .execute(pool)
            .await?;
    Ok(())
}

#[tracing::instrument(skip(user, pool, notifier))]
async fn get_project_doc_handler(
    Extension(user): Extension<User>,
    Extension(pool): Extension<&'static PgPool>,
    Extension(notifier): Extension<Notifier>,
    Path(project_id): Path<String>,
) -> ApiResult<Json<yrs::Any>> {
    verify_access(pool, user, &project_id).await?;

    Ok(Json(notifier.get_doc(&project_id).await?))
}
