use crate::{
    api::{
        ApiResult, IntoApiResult, bad_request_error, google::User, not_found_error,
        verify_project_access,
    },
    plugins::{
        config::ConfigStorage,
        google_workspace::{app::AppGoogleWorkspace, models::*},
    },
};
use anyhow::{Context, Result};
use axum::{
    Extension, Json, Router,
    extract::Query,
    routing::{delete, get, post},
};
use sqlx::PgPool;
use std::collections::HashMap;
use uuid::Uuid;

pub(crate) struct ConnectHandler {
    pool: &'static PgPool,
    config_storage: ConfigStorage,
    client: AppGoogleWorkspace,
}

impl ConnectHandler {
    pub(crate) fn new(
        pool: &'static PgPool,
        config_storage: ConfigStorage,
        client: AppGoogleWorkspace,
    ) -> Result<Self> {
        Ok(Self {
            pool,
            config_storage,
            client,
        })
    }

    pub(crate) fn router(self) -> Router {
        Router::new()
            .route("/connect", post(Self::connect_project_handler))
            .route("/disconnect", delete(Self::disconnect_project_handler))
            .route("/documents", get(Self::discover_documents_handler))
            .route("/documents/connect", post(Self::connect_document_handler))
            .route(
                "/documents/disconnect",
                delete(Self::disconnect_document_handler),
            )
            .route("/documents/sync", post(Self::sync_document_handler))
            .route(
                "/documents/list",
                get(Self::list_connected_documents_handler),
            )
            .layer(Extension(self))
    }

    /// Connect a project to Google Workspace
    #[tracing::instrument(
        skip(user, handler, request),
        fields(project_id=request.project_id)
    )]
    async fn connect_project_handler(
        Extension(user): Extension<User>,
        Extension(handler): Extension<ConnectHandler>,
        Json(request): Json<ConnectRequest>,
    ) -> ApiResult<Json<ConnectResponse>> {
        // Verify user has access to the project
        verify_project_access(handler.pool, &user, &request.project_id).await?;

        // Exchange authorization code for tokens
        let token_response = handler
            .client
            .exchange_code_for_tokens(&request.authorization_code, &request.redirect_uri)
            .await
            .context("Failed to exchange authorization code")?;

        // Get user info from Google
        let google_user = handler
            .client
            .get_user_info(&token_response.access_token)
            .await
            .context("Failed to get Google user info")?;

        // Store the connection
        let connection = GoogleWorkspaceConnection {
            id: Uuid::new_v4(),
            user_email: user.email.clone(),
            google_account_id: google_user.email.clone(),
            refresh_token: token_response.refresh_token.ok_or_else(|| {
                bad_request_error("No refresh token received", "No refresh token received")
            })?,
            access_token: Some(token_response.access_token),
            token_expires_at: token_response.expires_in.map(|expires_in| {
                chrono::Utc::now() + chrono::Duration::seconds(expires_in as i64)
            }),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        // Insert or update connection
        sqlx::query(
            "
            INSERT INTO google_workspace_connections 
            (id, user_email, google_account_id, refresh_token, access_token, token_expires_at, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT (user_email, google_account_id)
            DO UPDATE SET
                refresh_token = EXCLUDED.refresh_token,
                access_token = EXCLUDED.access_token,
                token_expires_at = EXCLUDED.token_expires_at,
                updated_at = EXCLUDED.updated_at
            "
        )
        .bind(&connection.id)
        .bind(&connection.user_email)
        .bind(&connection.google_account_id)
        .bind(&connection.refresh_token)
        .bind(&connection.access_token)
        .bind(&connection.token_expires_at)
        .bind(&connection.created_at)
        .bind(&connection.updated_at)
        .execute(handler.pool)
        .await
        .context("Failed to store Google Workspace connection")?;

        Ok(Json(ConnectResponse {
            success: true,
            message: "Successfully connected to Google Workspace".to_string(),
        }))
    }

    /// Disconnect a project from Google Workspace
    #[tracing::instrument(skip(user, handler), fields(project_id))]
    async fn disconnect_project_handler(
        Extension(user): Extension<User>,
        Extension(handler): Extension<ConnectHandler>,
        Query(params): Query<HashMap<String, String>>,
    ) -> ApiResult<Json<ConnectResponse>> {
        let project_id = params.get("project_id").ok_or_else(|| {
            bad_request_error(
                "Missing project_id parameter",
                "Missing project_id parameter",
            )
        })?;

        // Verify user has access to the project
        verify_project_access(handler.pool, &user, project_id).await?;

        // Remove all connected documents for this project
        sqlx::query("DELETE FROM google_workspace_documents WHERE project_id = $1")
            .bind(project_id)
            .execute(handler.pool)
            .await
            .context("Failed to remove connected documents")?;

        // Remove all Google Workspace tasks for this project
        sqlx::query("DELETE FROM google_workspace_tasks WHERE project_id = $1")
            .bind(project_id)
            .execute(handler.pool)
            .await
            .context("Failed to remove Google Workspace tasks")?;

        Ok(Json(ConnectResponse {
            success: true,
            message: "Successfully disconnected from Google Workspace".to_string(),
        }))
    }

    /// Discover available Google Workspace documents
    #[tracing::instrument(skip(user, handler), fields(project_id))]
    async fn discover_documents_handler(
        Extension(user): Extension<User>,
        Extension(handler): Extension<ConnectHandler>,
        Query(params): Query<HashMap<String, String>>,
    ) -> ApiResult<Json<DiscoverDocumentsResponse>> {
        let project_id = params.get("project_id").ok_or_else(|| {
            bad_request_error(
                "Missing project_id parameter",
                "Missing project_id parameter",
            )
        })?;

        // Verify user has access to the project
        verify_project_access(handler.pool, &user, project_id).await?;

        // Get user's Google Workspace connection
        let connection: GoogleWorkspaceConnection =
            sqlx::query_as("SELECT * FROM google_workspace_connections WHERE user_email = $1")
                .bind(&user.email)
                .fetch_optional(handler.pool)
                .await
                .context("Failed to fetch Google Workspace connection")?
                .ok_or_else(|| {
                    bad_request_error(
                        "No Google Workspace connection found",
                        "No Google Workspace connection found",
                    )
                })?;

        // Check if access token needs refresh
        let access_token = if connection
            .token_expires_at
            .map(|expires_at| expires_at <= chrono::Utc::now())
            .unwrap_or(false)
        {
            // Refresh token
            let token_response = handler
                .client
                .refresh_access_token(&connection.refresh_token)
                .await
                .context("Failed to refresh access token")?;

            // Update stored token
            sqlx::query(
                "UPDATE google_workspace_connections SET access_token = $1, token_expires_at = $2, updated_at = $3 WHERE id = $4"
            )
            .bind(&token_response.access_token)
            .bind(token_response.expires_in.map(|expires_in| {
                chrono::Utc::now() + chrono::Duration::seconds(expires_in as i64)
            }))
            .bind(&chrono::Utc::now())
            .bind(&connection.id)
            .execute(handler.pool)
            .await
            .context("Failed to update access token")?;

            token_response.access_token
        } else {
            connection.access_token.ok_or_else(|| {
                bad_request_error("No access token available", "No access token available")
            })?
        };

        // Get document types filter
        let document_types = params
            .get("document_types")
            .map(|types| types.split(',').map(|s| s.trim().to_string()).collect());

        // Discover documents
        let documents = handler
            .client
            .discover_documents(&access_token, document_types)
            .await
            .context("Failed to discover documents")?;

        // Get list of already connected documents
        let connected_documents: Vec<(String,)> = sqlx::query_as(
            "SELECT google_document_id FROM google_workspace_documents WHERE project_id = $1",
        )
        .bind(project_id)
        .fetch_all(handler.pool)
        .await
        .context("Failed to fetch connected documents")?;

        let connected_document_ids: Vec<String> =
            connected_documents.into_iter().map(|(id,)| id).collect();

        Ok(Json(DiscoverDocumentsResponse {
            documents,
            connected_document_ids,
        }))
    }

    /// Connect a specific document to a project
    #[tracing::instrument(
        skip(user, handler, request),
        fields(project_id=request.project_id, document_id=request.document_id)
    )]
    async fn connect_document_handler(
        Extension(user): Extension<User>,
        Extension(handler): Extension<ConnectHandler>,
        Json(request): Json<ConnectDocumentRequest>,
    ) -> ApiResult<Json<ConnectDocumentResponse>> {
        // Verify user has access to the project
        verify_project_access(handler.pool, &user, &request.project_id).await?;

        // Get user's Google Workspace connection
        let connection: GoogleWorkspaceConnection =
            sqlx::query_as("SELECT * FROM google_workspace_connections WHERE user_email = $1")
                .bind(&user.email)
                .fetch_optional(handler.pool)
                .await
                .context("Failed to fetch Google Workspace connection")?
                .ok_or_else(|| bad_request_error("No Google Workspace connection found"))?;

        // Check if access token needs refresh
        let access_token = if connection
            .token_expires_at
            .map(|expires_at| expires_at <= chrono::Utc::now())
            .unwrap_or(false)
        {
            // Refresh token
            let token_response = handler
                .client
                .refresh_access_token(&connection.refresh_token)
                .await
                .context("Failed to refresh access token")?;

            // Update stored token
            sqlx::query(
                "UPDATE google_workspace_connections SET access_token = $1, token_expires_at = $2, updated_at = $3 WHERE id = $4"
            )
            .bind(&token_response.access_token)
            .bind(token_response.expires_in.map(|expires_in| {
                chrono::Utc::now() + chrono::Duration::seconds(expires_in as i64)
            }))
            .bind(&chrono::Utc::now())
            .bind(&connection.id)
            .execute(handler.pool)
            .await
            .context("Failed to update access token")?;

            token_response.access_token
        } else {
            connection
                .access_token
                .ok_or_else(|| bad_request_error("No access token available"))?
        };

        // Get document info from Google Drive
        let documents = handler
            .client
            .discover_documents(&access_token, Some(vec![request.document_type.clone()]))
            .await
            .context("Failed to get document info")?;

        let document = documents
            .into_iter()
            .find(|doc| doc.id == request.document_id)
            .ok_or_else(|| not_found_error("Document not found"))?;

        // Store document connection
        sqlx::query(
            "
            INSERT INTO google_workspace_documents 
            (id, project_id, google_document_id, document_type, document_title, document_url, last_sync_at, sync_enabled, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            ON CONFLICT (project_id, google_document_id)
            DO UPDATE SET
                document_title = EXCLUDED.document_title,
                document_url = EXCLUDED.document_url,
                last_sync_at = EXCLUDED.last_sync_at,
                updated_at = NOW()
            "
        )
        .bind(&Uuid::new_v4())
        .bind(&request.project_id)
        .bind(&request.document_id)
        .bind(&request.document_type)
        .bind(&document.name)
        .bind(&document.web_view_link)
        .bind(&chrono::Utc::now())
        .bind(&true)
        .bind(&chrono::Utc::now())
        .execute(handler.pool)
        .await
        .context("Failed to store document connection")?;

        Ok(Json(ConnectDocumentResponse {
            success: true,
            message: "Successfully connected document".to_string(),
        }))
    }

    /// Disconnect a specific document from a project
    #[tracing::instrument(
        skip(user, handler, request),
        fields(project_id=request.project_id, document_id=request.document_id)
    )]
    async fn disconnect_document_handler(
        Extension(user): Extension<User>,
        Extension(handler): Extension<ConnectHandler>,
        Json(request): Json<DisconnectDocumentRequest>,
    ) -> ApiResult<Json<DisconnectDocumentResponse>> {
        // Verify user has access to the project
        verify_project_access(handler.pool, &user, &request.project_id).await?;

        // Remove document connection
        sqlx::query(
            "DELETE FROM google_workspace_documents WHERE project_id = $1 AND google_document_id = $2"
        )
        .bind(&request.project_id)
        .bind(&request.document_id)
        .execute(handler.pool)
        .await
        .context("Failed to remove document connection")?;

        // Remove associated Google Workspace tasks
        sqlx::query(
            "DELETE FROM google_workspace_tasks WHERE project_id = $1 AND google_document_id = $2",
        )
        .bind(&request.project_id)
        .bind(&request.document_id)
        .execute(handler.pool)
        .await
        .context("Failed to remove Google Workspace tasks")?;

        Ok(Json(DisconnectDocumentResponse {
            success: true,
            message: "Successfully disconnected document".to_string(),
        }))
    }

    /// Manually sync a specific document
    #[tracing::instrument(
        skip(user, handler, request),
        fields(project_id=request.project_id, document_id=request.document_id)
    )]
    async fn sync_document_handler(
        Extension(user): Extension<User>,
        Extension(handler): Extension<ConnectHandler>,
        Json(request): Json<SyncDocumentRequest>,
    ) -> ApiResult<Json<SyncDocumentResponse>> {
        // Verify user has access to the project
        verify_project_access(handler.pool, &user, &request.project_id).await?;

        // Get document info
        let document: GoogleWorkspaceDocument = sqlx::query_as(
            "SELECT * FROM google_workspace_documents WHERE project_id = $1 AND google_document_id = $2"
        )
        .bind(&request.project_id)
        .bind(&request.document_id)
        .fetch_optional(handler.pool)
        .await
        .context("Failed to fetch document")?
        .ok_or_else(|| not_found_error("Document not found"))?;

        // Get user's Google Workspace connection
        let connection: GoogleWorkspaceConnection =
            sqlx::query_as("SELECT * FROM google_workspace_connections WHERE user_email = $1")
                .bind(&user.email)
                .fetch_optional(handler.pool)
                .await
                .context("Failed to fetch Google Workspace connection")?
                .ok_or_else(|| bad_request_error("No Google Workspace connection found"))?;

        // Check if access token needs refresh
        let access_token = if connection
            .token_expires_at
            .map(|expires_at| expires_at <= chrono::Utc::now())
            .unwrap_or(false)
        {
            // Refresh token
            let token_response = handler
                .client
                .refresh_access_token(&connection.refresh_token)
                .await
                .context("Failed to refresh access token")?;

            // Update stored token
            sqlx::query(
                "UPDATE google_workspace_connections SET access_token = $1, token_expires_at = $2, updated_at = $3 WHERE id = $4"
            )
            .bind(&token_response.access_token)
            .bind(token_response.expires_in.map(|expires_in| {
                chrono::Utc::now() + chrono::Duration::seconds(expires_in as i64)
            }))
            .bind(&chrono::Utc::now())
            .bind(&connection.id)
            .execute(handler.pool)
            .await
            .context("Failed to update access token")?;

            token_response.access_token
        } else {
            connection
                .access_token
                .ok_or_else(|| bad_request_error("No access token available"))?
        };

        // TODO: Implement actual sync logic
        // This would involve:
        // 1. Getting comments and reviewers from the document
        // 2. Creating/updating tasks in Koso
        // 3. Updating the sync timestamp

        // Update last sync time
        sqlx::query("UPDATE google_workspace_documents SET last_sync_at = $1 WHERE id = $2")
            .bind(&chrono::Utc::now())
            .bind(&document.id)
            .execute(handler.pool)
            .await
            .context("Failed to update sync time")?;

        Ok(Json(SyncDocumentResponse {
            success: true,
            message: "Document sync completed".to_string(),
            tasks_created: 0, // TODO: Implement actual counting
            tasks_updated: 0, // TODO: Implement actual counting
        }))
    }

    /// List all connected documents for a project
    #[tracing::instrument(skip(user, handler), fields(project_id))]
    async fn list_connected_documents_handler(
        Extension(user): Extension<User>,
        Extension(handler): Extension<ConnectHandler>,
        Query(params): Query<HashMap<String, String>>,
    ) -> ApiResult<Json<Vec<GoogleWorkspaceDocument>>> {
        let project_id = params
            .get("project_id")
            .ok_or_else(|| bad_request_error("Missing project_id parameter"))?;

        // Verify user has access to the project
        verify_project_access(handler.pool, &user, project_id).await?;

        // Get connected documents
        let documents: Vec<GoogleWorkspaceDocument> = sqlx::query_as(
            "SELECT * FROM google_workspace_documents WHERE project_id = $1 ORDER BY created_at DESC"
        )
        .bind(project_id)
        .fetch_all(handler.pool)
        .await
        .context("Failed to fetch connected documents")?;

        Ok(Json(documents))
    }
}
