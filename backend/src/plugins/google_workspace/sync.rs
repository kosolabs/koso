use crate::{
    api::{
        collab::Collab,
        model::Task,
        yproxy::{YDocProxy, YTaskProxy},
    },
    plugins::{
        config::ConfigStorage,
        google_workspace::{app::AppGoogleWorkspace, models::*},
    },
};
use anyhow::{Context, Result};
use axum::{Router, extract::Extension, routing::post};
use base64::{Engine as _, prelude::BASE64_URL_SAFE_NO_PAD};
use sqlx::PgPool;
use std::time::Duration;
use tokio::time::sleep;
use uuid::Uuid;
use yrs::{Doc, TransactionMut};

#[derive(Clone)]
pub(crate) struct SyncService {
    collab: Collab,
    client: AppGoogleWorkspace,
    config_storage: ConfigStorage,
    pool: &'static PgPool,
}

impl SyncService {
    pub(crate) fn new(
        collab: Collab,
        client: AppGoogleWorkspace,
        config_storage: ConfigStorage,
        pool: &'static PgPool,
    ) -> Self {
        Self {
            collab,
            client,
            config_storage,
            pool,
        }
    }

    /// Start the background sync service
    pub(crate) async fn run(self) {
        let mut interval = tokio::time::interval(Duration::from_secs(15 * 60)); // 15 minutes

        loop {
            interval.tick().await;

            if let Err(e) = self.sync_all_documents().await {
                tracing::error!("Failed to sync Google Workspace documents: {:?}", e);
            }
        }
    }

    /// Get router for manual sync endpoints
    pub(crate) fn router(self) -> Result<Router> {
        Ok(Router::new()
            .route("/sync/all", post(Self::manual_sync_all_handler))
            .route("/sync/document", post(Self::manual_sync_document_handler))
            .layer(Extension(self)))
    }

    /// Manual sync all documents handler
    async fn manual_sync_all_handler(
        Extension(service): Extension<SyncService>,
    ) -> axum::Json<serde_json::Value> {
        match service.sync_all_documents().await {
            Ok(_) => axum::Json(serde_json::json!({
                "success": true,
                "message": "Sync completed successfully"
            })),
            Err(e) => axum::Json(serde_json::json!({
                "success": false,
                "message": format!("Sync failed: {:?}", e)
            })),
        }
    }

    /// Manual sync specific document handler
    async fn manual_sync_document_handler(
        Extension(service): Extension<SyncService>,
        axum::Json(request): axum::Json<SyncDocumentRequest>,
    ) -> axum::Json<serde_json::Value> {
        match service
            .sync_document(&request.project_id, &request.document_id)
            .await
        {
            Ok(_) => axum::Json(serde_json::json!({
                "success": true,
                "message": "Document sync completed successfully"
            })),
            Err(e) => axum::Json(serde_json::json!({
                "success": false,
                "message": format!("Document sync failed: {:?}", e)
            })),
        }
    }

    /// Sync all documents that need syncing
    async fn sync_all_documents(&self) -> Result<()> {
        tracing::info!("Starting Google Workspace document sync");

        // Get all documents that need syncing
        let documents: Vec<GoogleWorkspaceDocument> = sqlx::query_as(
            "
            SELECT * FROM google_workspace_documents 
            WHERE sync_enabled = true 
            AND (last_sync_at IS NULL OR last_sync_at < NOW() - INTERVAL '15 minutes')
            ORDER BY last_sync_at ASC NULLS FIRST
            LIMIT 10
            ",
        )
        .fetch_all(self.pool)
        .await
        .context("Failed to fetch documents for sync")?;

        tracing::info!("Found {} documents to sync", documents.len());

        for document in documents {
            if let Err(e) = self
                .sync_document(&document.project_id, &document.google_document_id)
                .await
            {
                tracing::error!(
                    "Failed to sync document {}: {:?}",
                    document.google_document_id,
                    e
                );
                // Continue with other documents
            }
        }

        tracing::info!("Google Workspace document sync completed");
        Ok(())
    }

    /// Sync a specific document
    async fn sync_document(&self, project_id: &str, document_id: &str) -> Result<()> {
        tracing::debug!(
            "Syncing document {} for project {}",
            document_id,
            project_id
        );

        // Get document info
        let document: GoogleWorkspaceDocument = sqlx::query_as(
            "SELECT * FROM google_workspace_documents WHERE project_id = $1 AND google_document_id = $2"
        )
        .bind(project_id)
        .bind(document_id)
        .fetch_optional(self.pool)
        .await
        .context("Failed to fetch document")?
        .ok_or_else(|| anyhow::anyhow!("Document not found"))?;

        // Get user connection for this document
        let connection: GoogleWorkspaceConnection = sqlx::query_as(
            "
            SELECT c.* FROM google_workspace_connections c
            JOIN google_workspace_documents d ON d.project_id = $1
            WHERE d.google_document_id = $2
            LIMIT 1
            ",
        )
        .bind(project_id)
        .bind(document_id)
        .fetch_optional(self.pool)
        .await
        .context("Failed to fetch connection")?
        .ok_or_else(|| anyhow::anyhow!("No connection found for document"))?;

        // Check if access token needs refresh
        let access_token = if connection
            .token_expires_at
            .map(|expires_at| expires_at <= chrono::Utc::now())
            .unwrap_or(false)
        {
            // Refresh token
            let token_response = self
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
            .execute(self.pool)
            .await
            .context("Failed to update access token")?;

            token_response.access_token
        } else {
            connection
                .access_token
                .ok_or_else(|| anyhow::anyhow!("No access token available"))?
        };

        // Get comments from the document
        let comments = self
            .client
            .get_document_comments(&access_token, document_id, &document.document_type)
            .await
            .context("Failed to get document comments")?;

        // Get reviewers from the document
        let reviewers = self
            .client
            .get_document_reviewers(&access_token, document_id, &document.document_type)
            .await
            .context("Failed to get document reviewers")?;

        // Sync comments
        self.sync_comments(project_id, document_id, &comments)
            .await?;

        // Sync reviewers
        self.sync_reviewers(project_id, document_id, &reviewers)
            .await?;

        // Update last sync time
        sqlx::query("UPDATE google_workspace_documents SET last_sync_at = $1 WHERE id = $2")
            .bind(&chrono::Utc::now())
            .bind(&document.id)
            .execute(self.pool)
            .await
            .context("Failed to update sync time")?;

        tracing::debug!("Successfully synced document {}", document_id);
        Ok(())
    }

    /// Sync comments from Google Workspace to Koso tasks
    async fn sync_comments(
        &self,
        project_id: &str,
        document_id: &str,
        comments: &[GoogleComment],
    ) -> Result<()> {
        for comment in comments {
            // Check if we already have a task for this comment
            let existing_task: Option<GoogleWorkspaceTask> = sqlx::query_as(
                "
                SELECT * FROM google_workspace_tasks 
                WHERE project_id = $1 AND google_document_id = $2 AND google_comment_id = $3
                ",
            )
            .bind(project_id)
            .bind(document_id)
            .bind(&comment.id)
            .fetch_optional(self.pool)
            .await
            .context("Failed to check existing comment task")?;

            if let Some(existing_task) = existing_task {
                // Update existing task
                self.update_comment_task(existing_task, comment).await?;
            } else {
                // Create new task
                self.create_comment_task(project_id, document_id, comment)
                    .await?;
            }
        }

        Ok(())
    }

    /// Sync reviewers from Google Workspace to Koso tasks
    async fn sync_reviewers(
        &self,
        project_id: &str,
        document_id: &str,
        reviewers: &[GoogleReviewer],
    ) -> Result<()> {
        for reviewer in reviewers {
            // Check if we already have a task for this reviewer
            let existing_task: Option<GoogleWorkspaceTask> = sqlx::query_as(
                "
                SELECT * FROM google_workspace_tasks 
                WHERE project_id = $1 AND google_document_id = $2 AND google_reviewer_id = $3
                ",
            )
            .bind(project_id)
            .bind(document_id)
            .bind(&reviewer.id)
            .fetch_optional(self.pool)
            .await
            .context("Failed to check existing reviewer task")?;

            if let Some(existing_task) = existing_task {
                // Update existing task
                self.update_reviewer_task(existing_task, reviewer).await?;
            } else {
                // Create new task
                self.create_reviewer_task(project_id, document_id, reviewer)
                    .await?;
            }
        }

        Ok(())
    }

    /// Create a new task for a Google comment
    async fn create_comment_task(
        &self,
        project_id: &str,
        document_id: &str,
        comment: &GoogleComment,
    ) -> Result<()> {
        // Create Koso task
        let task_id = self.create_koso_task(project_id, comment).await?;

        // Store Google Workspace task reference
        let google_task = GoogleWorkspaceTask {
            id: Uuid::new_v4(),
            project_id: project_id.to_string(),
            google_document_id: document_id.to_string(),
            google_comment_id: Some(comment.id.clone()),
            google_reviewer_id: None,
            task_id: Some(task_id),
            task_type: "comment".to_string(),
            status: if comment.is_resolved() {
                "resolved".to_string()
            } else {
                "pending".to_string()
            },
            assignee_email: Some(comment.author.email.clone()),
            due_date: None,
            last_sync_at: chrono::Utc::now(),
            created_at: chrono::Utc::now(),
        };

        sqlx::query(
            "
            INSERT INTO google_workspace_tasks 
            (id, project_id, google_document_id, google_comment_id, google_reviewer_id, task_id, task_type, status, assignee_email, due_date, last_sync_at, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            "
        )
        .bind(&google_task.id)
        .bind(&google_task.project_id)
        .bind(&google_task.google_document_id)
        .bind(&google_task.google_comment_id)
        .bind(&google_task.google_reviewer_id)
        .bind(&google_task.task_id)
        .bind(&google_task.task_type)
        .bind(&google_task.status)
        .bind(&google_task.assignee_email)
        .bind(&google_task.due_date)
        .bind(&google_task.last_sync_at)
        .bind(&google_task.created_at)
        .execute(self.pool)
        .await
        .context("Failed to store Google Workspace task")?;

        Ok(())
    }

    /// Create a new task for a Google reviewer
    async fn create_reviewer_task(
        &self,
        project_id: &str,
        document_id: &str,
        reviewer: &GoogleReviewer,
    ) -> Result<()> {
        // Create Koso task
        let task_id = self.create_koso_reviewer_task(project_id, reviewer).await?;

        // Store Google Workspace task reference
        let google_task = GoogleWorkspaceTask {
            id: Uuid::new_v4(),
            project_id: project_id.to_string(),
            google_document_id: document_id.to_string(),
            google_comment_id: None,
            google_reviewer_id: Some(reviewer.id.clone()),
            task_id: Some(task_id),
            task_type: "reviewer".to_string(),
            status: if reviewer.is_pending() {
                "pending".to_string()
            } else {
                "resolved".to_string()
            },
            assignee_email: Some(reviewer.email.clone()),
            due_date: reviewer.due_date,
            last_sync_at: chrono::Utc::now(),
            created_at: chrono::Utc::now(),
        };

        sqlx::query(
            "
            INSERT INTO google_workspace_tasks 
            (id, project_id, google_document_id, google_comment_id, google_reviewer_id, task_id, task_type, status, assignee_email, due_date, last_sync_at, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            "
        )
        .bind(&google_task.id)
        .bind(&google_task.project_id)
        .bind(&google_task.google_document_id)
        .bind(&google_task.google_comment_id)
        .bind(&google_task.google_reviewer_id)
        .bind(&google_task.task_id)
        .bind(&google_task.task_type)
        .bind(&google_task.status)
        .bind(&google_task.assignee_email)
        .bind(&google_task.due_date)
        .bind(&google_task.last_sync_at)
        .bind(&google_task.created_at)
        .execute(self.pool)
        .await
        .context("Failed to store Google Workspace task")?;

        Ok(())
    }

    /// Update an existing comment task
    async fn update_comment_task(
        &self,
        mut existing_task: GoogleWorkspaceTask,
        comment: &GoogleComment,
    ) -> Result<()> {
        // Update status
        existing_task.status = if comment.is_resolved() {
            "resolved".to_string()
        } else {
            "pending".to_string()
        };
        existing_task.last_sync_at = chrono::Utc::now();

        // Update in database
        sqlx::query(
            "
            UPDATE google_workspace_tasks 
            SET status = $1, last_sync_at = $2 
            WHERE id = $3
            ",
        )
        .bind(&existing_task.status)
        .bind(&existing_task.last_sync_at)
        .bind(&existing_task.id)
        .execute(self.pool)
        .await
        .context("Failed to update Google Workspace task")?;

        // Update Koso task if it exists
        if let Some(task_id) = &existing_task.task_id {
            self.update_koso_task_status(task_id, &existing_task.status)
                .await?;
        }

        Ok(())
    }

    /// Update an existing reviewer task
    async fn update_reviewer_task(
        &self,
        mut existing_task: GoogleWorkspaceTask,
        reviewer: &GoogleReviewer,
    ) -> Result<()> {
        // Update status
        existing_task.status = if reviewer.is_pending() {
            "pending".to_string()
        } else {
            "resolved".to_string()
        };
        existing_task.last_sync_at = chrono::Utc::now();

        // Update in database
        sqlx::query(
            "
            UPDATE google_workspace_tasks 
            SET status = $1, last_sync_at = $2 
            WHERE id = $3
            ",
        )
        .bind(&existing_task.status)
        .bind(&existing_task.last_sync_at)
        .bind(&existing_task.id)
        .execute(self.pool)
        .await
        .context("Failed to update Google Workspace task")?;

        // Update Koso task if it exists
        if let Some(task_id) = &existing_task.task_id {
            self.update_koso_task_status(task_id, &existing_task.status)
                .await?;
        }

        Ok(())
    }

    /// Create a Koso task for a comment
    async fn create_koso_task(&self, project_id: &str, comment: &GoogleComment) -> Result<String> {
        let client = self.collab.register_local_client(project_id).await?;

        let doc = client.project.doc_box.lock().await;
        let doc = crate::api::collab::projects_state::DocBox::doc_or_error(doc.as_ref())?;
        let doc = &doc.ydoc;
        let mut txn = doc.transact_mut();

        let id = BASE64_URL_SAFE_NO_PAD.encode(uuid::Uuid::new_v4());
        let num = doc.next_num(&txn)?.to_string();

        let parent = doc.get(&txn, "root")?;
        let mut children: Vec<String> = parent.get_children(&txn)?;

        let task = doc.set(
            &mut txn,
            &Task {
                id: id.clone(),
                num,
                name: format!("Review comment: {}", comment.summary()),
                desc: Some(comment.content.clone()),
                children: Vec::new(),
                assignee: Some(comment.author.email.clone()),
                reporter: Some(comment.author.email.clone()),
                status: Some(if comment.is_resolved() {
                    "Done".to_string()
                } else {
                    "In Progress".to_string()
                }),
                status_time: Some(chrono::Utc::now().timestamp()),
                url: Some(comment.document_url.clone()),
                kind: Some("google_comment".to_string()),
                estimate: None,
                deadline: None,
                archived: Some(false),
            },
        );

        children.push(id.clone());
        parent.set_children(&mut txn, &children);

        Ok(id)
    }

    /// Create a Koso task for a reviewer
    async fn create_koso_reviewer_task(
        &self,
        project_id: &str,
        reviewer: &GoogleReviewer,
    ) -> Result<String> {
        let client = self.collab.register_local_client(project_id).await?;

        let doc = client.project.doc_box.lock().await;
        let doc = crate::api::collab::projects_state::DocBox::doc_or_error(doc.as_ref())?;
        let doc = &doc.ydoc;
        let mut txn = doc.transact_mut();

        let id = BASE64_URL_SAFE_NO_PAD.encode(uuid::Uuid::new_v4());
        let num = doc.next_num(&txn)?.to_string();

        let parent = doc.get(&txn, "root")?;
        let mut children: Vec<String> = parent.get_children(&txn)?;

        let task = doc.set(
            &mut txn,
            &Task {
                id: id.clone(),
                num,
                name: format!("Review document: {}", reviewer.document_title),
                desc: Some(format!("Required review by {}", reviewer.email)),
                children: Vec::new(),
                assignee: Some(reviewer.email.clone()),
                reporter: Some(reviewer.requestor_email.clone()),
                status: Some(if reviewer.is_pending() {
                    "Pending Review".to_string()
                } else {
                    "Done".to_string()
                }),
                status_time: Some(chrono::Utc::now().timestamp()),
                url: Some(reviewer.document_url.clone()),
                kind: Some("google_reviewer".to_string()),
                estimate: None,
                deadline: reviewer.due_date.map(|d| d.timestamp()),
                archived: Some(false),
            },
        );

        children.push(id.clone());
        parent.set_children(&mut txn, &children);

        Ok(id)
    }

    /// Update Koso task status
    async fn update_koso_task_status(&self, task_id: &str, status: &str) -> Result<()> {
        // TODO: Implement task status update in Koso
        // This would require finding the task in the project and updating its status
        tracing::debug!("Would update Koso task {} status to {}", task_id, status);
        Ok(())
    }
}
