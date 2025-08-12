use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use sqlx::types::chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GoogleWorkspaceConnection {
    pub(crate) id: uuid::Uuid,
    pub(crate) user_email: String,
    pub(crate) google_account_id: String,
    pub(crate) refresh_token: String,
    pub(crate) access_token: Option<String>,
    pub(crate) token_expires_at: Option<DateTime<Utc>>,
    pub(crate) created_at: DateTime<Utc>,
    pub(crate) updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GoogleWorkspaceDocument {
    pub(crate) id: uuid::Uuid,
    pub(crate) project_id: String,
    pub(crate) google_document_id: String,
    pub(crate) document_type: String, // 'docs', 'sheets', 'slides'
    pub(crate) document_title: String,
    pub(crate) document_url: String,
    pub(crate) last_sync_at: DateTime<Utc>,
    pub(crate) sync_enabled: bool,
    pub(crate) created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GoogleWorkspaceTask {
    pub(crate) id: uuid::Uuid,
    pub(crate) project_id: String,
    pub(crate) google_document_id: String,
    pub(crate) google_comment_id: Option<String>,
    pub(crate) google_reviewer_id: Option<String>,
    pub(crate) task_id: Option<String>, // References Koso task
    pub(crate) task_type: String,       // 'comment', 'reviewer'
    pub(crate) status: String,          // 'pending', 'resolved', 'archived'
    pub(crate) assignee_email: Option<String>,
    pub(crate) due_date: Option<DateTime<Utc>>,
    pub(crate) last_sync_at: DateTime<Utc>,
    pub(crate) created_at: DateTime<Utc>,
}

// Google API Models
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GoogleComment {
    pub(crate) id: String,
    pub(crate) content: String,
    pub(crate) author: GoogleUser,
    pub(crate) status: String, // 'open', 'resolved'
    pub(crate) created_time: DateTime<Utc>,
    pub(crate) modified_time: DateTime<Utc>,
    pub(crate) resolved_time: Option<DateTime<Utc>>,
    pub(crate) document_id: String,
    pub(crate) document_type: String,
    pub(crate) document_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GoogleReviewer {
    pub(crate) id: String,
    pub(crate) email: String,
    pub(crate) name: String,
    pub(crate) status: String, // 'pending', 'approved', 'rejected'
    pub(crate) requested_time: DateTime<Utc>,
    pub(crate) responded_time: Option<DateTime<Utc>>,
    pub(crate) document_id: String,
    pub(crate) document_type: String,
    pub(crate) document_title: String,
    pub(crate) document_url: String,
    pub(crate) due_date: Option<DateTime<Utc>>,
    pub(crate) requestor_email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GoogleUser {
    pub(crate) email: String,
    pub(crate) name: String,
    pub(crate) picture: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GoogleDocument {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) mime_type: String,
    pub(crate) web_view_link: String,
    pub(crate) created_time: DateTime<Utc>,
    pub(crate) modified_time: DateTime<Utc>,
    pub(crate) owners: Vec<GoogleUser>,
    pub(crate) writers: Vec<GoogleUser>,
    pub(crate) readers: Vec<GoogleUser>,
}

// API Request/Response Models
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ConnectRequest {
    pub(crate) project_id: String,
    pub(crate) authorization_code: String,
    pub(crate) redirect_uri: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ConnectResponse {
    pub(crate) success: bool,
    pub(crate) message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DiscoverDocumentsRequest {
    pub(crate) project_id: String,
    pub(crate) document_types: Option<Vec<String>>, // ['docs', 'sheets', 'slides']
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DiscoverDocumentsResponse {
    pub(crate) documents: Vec<GoogleDocument>,
    pub(crate) connected_document_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ConnectDocumentRequest {
    pub(crate) project_id: String,
    pub(crate) document_id: String,
    pub(crate) document_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ConnectDocumentResponse {
    pub(crate) success: bool,
    pub(crate) message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DisconnectDocumentRequest {
    pub(crate) project_id: String,
    pub(crate) document_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DisconnectDocumentResponse {
    pub(crate) success: bool,
    pub(crate) message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SyncDocumentRequest {
    pub(crate) project_id: String,
    pub(crate) document_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SyncDocumentResponse {
    pub(crate) success: bool,
    pub(crate) message: String,
    pub(crate) tasks_created: usize,
    pub(crate) tasks_updated: usize,
}

// Helper functions
impl GoogleComment {
    pub(crate) fn is_resolved(&self) -> bool {
        self.status == "resolved" || self.resolved_time.is_some()
    }

    pub(crate) fn summary(&self) -> String {
        if self.content.len() > 100 {
            format!("{}...", &self.content[..97])
        } else {
            self.content.clone()
        }
    }
}

impl GoogleReviewer {
    pub(crate) fn is_pending(&self) -> bool {
        self.status == "pending"
    }

    pub(crate) fn is_approved(&self) -> bool {
        self.status == "approved"
    }

    pub(crate) fn is_rejected(&self) -> bool {
        self.status == "rejected"
    }
}

impl GoogleDocument {
    pub(crate) fn document_type(&self) -> String {
        match self.mime_type.as_str() {
            "application/vnd.google-apps.document" => "docs".to_string(),
            "application/vnd.google-apps.spreadsheet" => "sheets".to_string(),
            "application/vnd.google-apps.presentation" => "slides".to_string(),
            _ => "unknown".to_string(),
        }
    }

    pub(crate) fn is_google_workspace_document(&self) -> bool {
        matches!(
            self.mime_type.as_str(),
            "application/vnd.google-apps.document"
                | "application/vnd.google-apps.spreadsheet"
                | "application/vnd.google-apps.presentation"
        )
    }
}
