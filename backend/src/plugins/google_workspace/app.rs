use crate::{
    api::ApiResult,
    plugins::google_workspace::models::{
        GoogleComment, GoogleDocument, GoogleReviewer, GoogleUser,
    },
    secrets::read_secret,
};
use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sqlx::types::chrono::{DateTime, Utc};
use std::time::Duration;

#[derive(Clone)]
pub(crate) struct AppGoogleWorkspace {
    client: Client,
    client_id: String,
    client_secret: String,
}

impl AppGoogleWorkspace {
    pub(crate) async fn new() -> Result<Self> {
        let client_id = read_secret("google_workspace/client_id")?;
        let client_secret = read_secret("google_workspace/client_secret")?;

        let client = Client::builder().timeout(Duration::from_secs(30)).build()?;

        Ok(Self {
            client,
            client_id,
            client_secret,
        })
    }

    /// Exchange authorization code for access and refresh tokens
    pub(crate) async fn exchange_code_for_tokens(
        &self,
        authorization_code: &str,
        redirect_uri: &str,
    ) -> Result<TokenResponse> {
        let params = [
            ("client_id", &self.client_id),
            ("client_secret", &self.client_secret),
            ("code", &authorization_code.to_string()),
            ("grant_type", &"authorization_code".to_string()),
            ("redirect_uri", &redirect_uri.to_string()),
        ];

        let response = self
            .client
            .post("https://oauth2.googleapis.com/token")
            .form(&params)
            .send()
            .await
            .context("Failed to exchange code for tokens")?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow::anyhow!("Token exchange failed: {}", error_text));
        }

        let token_response: TokenResponse = response
            .json()
            .await
            .context("Failed to parse token response")?;

        Ok(token_response)
    }

    /// Refresh access token using refresh token
    pub(crate) async fn refresh_access_token(&self, refresh_token: &str) -> Result<TokenResponse> {
        let params = [
            ("client_id", &self.client_id),
            ("client_secret", &self.client_secret),
            ("refresh_token", &refresh_token.to_string()),
            ("grant_type", &"refresh_token".to_string()),
        ];

        let response = self
            .client
            .post("https://oauth2.googleapis.com/token")
            .form(&params)
            .send()
            .await
            .context("Failed to refresh access token")?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow::anyhow!("Token refresh failed: {}", error_text));
        }

        let token_response: TokenResponse = response
            .json()
            .await
            .context("Failed to parse token refresh response")?;

        Ok(token_response)
    }

    /// Get user info from Google
    pub(crate) async fn get_user_info(&self, access_token: &str) -> Result<GoogleUser> {
        let response = self
            .client
            .get("https://www.googleapis.com/oauth2/v2/userinfo")
            .header("Authorization", format!("Bearer {}", access_token))
            .send()
            .await
            .context("Failed to get user info")?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!("Failed to get user info"));
        }

        let user_info: GoogleUserInfo =
            response.json().await.context("Failed to parse user info")?;

        Ok(GoogleUser {
            email: user_info.email,
            name: user_info.name,
            picture: Some(user_info.picture),
        })
    }

    /// Discover Google Workspace documents accessible to the user
    pub(crate) async fn discover_documents(
        &self,
        access_token: &str,
        document_types: Option<Vec<String>>,
    ) -> Result<Vec<GoogleDocument>> {
        let mut documents = Vec::new();

        // Get documents from Google Drive
        let drive_docs = self
            .get_drive_documents(access_token, document_types.as_ref())
            .await?;
        documents.extend(drive_docs);

        Ok(documents)
    }

    /// Get documents from Google Drive
    async fn get_drive_documents(
        &self,
        access_token: &str,
        document_types: Option<&Vec<String>>,
    ) -> Result<Vec<GoogleDocument>> {
        let mut query =
            "mimeType contains 'application/vnd.google-apps' and trashed=false".to_string();

        if let Some(types) = document_types {
            let type_filters: Vec<String> = types
                .iter()
                .map(|t| match t.as_str() {
                    "docs" => "mimeType='application/vnd.google-apps.document'".to_string(),
                    "sheets" => "mimeType='application/vnd.google-apps.spreadsheet'".to_string(),
                    "slides" => "mimeType='application/vnd.google-apps.presentation'".to_string(),
                    _ => format!("mimeType='{}'", t),
                })
                .collect();

            if !type_filters.is_empty() {
                query = format!("({}) and ({})", query, type_filters.join(" or "));
            }
        }

        let url = format!(
            "https://www.googleapis.com/drive/v3/files?q={}&fields=files(id,name,mimeType,webViewLink,createdTime,modifiedTime,owners,writers,readers)&orderBy=modifiedTime desc&pageSize=100",
            urlencoding::encode(&query)
        );

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", access_token))
            .send()
            .await
            .context("Failed to get drive documents")?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!("Failed to get drive documents"));
        }

        let drive_response: DriveResponse = response
            .json()
            .await
            .context("Failed to parse drive response")?;

        let documents: Result<Vec<GoogleDocument>> = drive_response
            .files
            .into_iter()
            .filter(|file| file.is_google_workspace_document())
            .map(|file| {
                let created_time = DateTime::parse_from_rfc3339(&file.created_time)
                    .unwrap_or_else(|_| Utc::now())
                    .with_timezone(&Utc);
                let modified_time = DateTime::parse_from_rfc3339(&file.modified_time)
                    .unwrap_or_else(|_| Utc::now())
                    .with_timezone(&Utc);

                Ok(GoogleDocument {
                    id: file.id,
                    name: file.name,
                    mime_type: file.mime_type,
                    web_view_link: file.web_view_link,
                    created_time,
                    modified_time,
                    owners: file.owners.unwrap_or_default(),
                    writers: file.writers.unwrap_or_default(),
                    readers: file.readers.unwrap_or_default(),
                })
            })
            .collect();

        documents
    }

    /// Get comments from a Google Document
    pub(crate) async fn get_document_comments(
        &self,
        access_token: &str,
        document_id: &str,
        document_type: &str,
    ) -> Result<Vec<GoogleComment>> {
        match document_type {
            "docs" => self.get_docs_comments(access_token, document_id).await,
            "sheets" => self.get_sheets_comments(access_token, document_id).await,
            "slides" => self.get_slides_comments(access_token, document_id).await,
            _ => Ok(Vec::new()),
        }
    }

    /// Get comments from Google Docs
    async fn get_docs_comments(
        &self,
        access_token: &str,
        document_id: &str,
    ) -> Result<Vec<GoogleComment>> {
        let url = format!(
            "https://docs.googleapis.com/v1/documents/{}/comments",
            document_id
        );

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", access_token))
            .send()
            .await
            .context("Failed to get docs comments")?;

        if !response.status().is_success() {
            return Ok(Vec::new()); // No comments or document not accessible
        }

        let comments_response: DocsCommentsResponse = response
            .json()
            .await
            .context("Failed to parse docs comments response")?;

        let comments: Result<Vec<GoogleComment>> = comments_response
            .comments
            .unwrap_or_default()
            .into_iter()
            .map(|comment| {
                let created_time = DateTime::parse_from_rfc3339(&comment.created_time)
                    .unwrap_or_else(|_| Utc::now())
                    .with_timezone(&Utc);
                let modified_time = DateTime::parse_from_rfc3339(&comment.modified_time)
                    .unwrap_or_else(|_| Utc::now())
                    .with_timezone(&Utc);

                Ok(GoogleComment {
                    id: comment.id,
                    content: comment.content.unwrap_or_default(),
                    author: GoogleUser {
                        email: comment.author.email.unwrap_or_default(),
                        name: comment.author.name.unwrap_or_default(),
                        picture: comment.author.picture,
                    },
                    status: comment.status.unwrap_or_else(|| "open".to_string()),
                    created_time,
                    modified_time,
                    resolved_time: comment.resolved_time.and_then(|t| {
                        DateTime::parse_from_rfc3339(&t)
                            .ok()
                            .map(|dt| dt.with_timezone(&Utc))
                    }),
                    document_id: document_id.to_string(),
                    document_type: "docs".to_string(),
                    document_url: format!("https://docs.google.com/document/d/{}", document_id),
                })
            })
            .collect();

        comments
    }

    /// Get comments from Google Sheets
    async fn get_sheets_comments(
        &self,
        access_token: &str,
        _document_id: &str,
    ) -> Result<Vec<GoogleComment>> {
        // TODO: Implement sheets comments API
        // Sheets comments are more complex and require different API endpoints
        Ok(Vec::new())
    }

    /// Get comments from Google Slides
    async fn get_slides_comments(
        &self,
        access_token: &str,
        _document_id: &str,
    ) -> Result<Vec<GoogleComment>> {
        // TODO: Implement slides comments API
        // Slides comments require different API endpoints
        Ok(Vec::new())
    }

    /// Get reviewers from a Google Document
    pub(crate) async fn get_document_reviewers(
        &self,
        access_token: &str,
        document_id: &str,
        document_type: &str,
    ) -> Result<Vec<GoogleReviewer>> {
        // TODO: Implement reviewer extraction
        // This will require parsing document permissions and sharing settings
        Ok(Vec::new())
    }
}

// API Response Models
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: Option<i32>,
    pub refresh_token: Option<String>,
    pub scope: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GoogleUserInfo {
    pub id: String,
    pub email: String,
    pub verified_email: bool,
    pub name: String,
    pub given_name: String,
    pub family_name: String,
    pub picture: String,
    pub locale: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DriveResponse {
    pub files: Vec<DriveFile>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DriveFile {
    pub id: String,
    pub name: String,
    pub mime_type: String,
    pub web_view_link: String,
    pub created_time: String,
    pub modified_time: String,
    pub owners: Option<Vec<GoogleUser>>,
    pub writers: Option<Vec<GoogleUser>>,
    pub readers: Option<Vec<GoogleUser>>,
}

impl DriveFile {
    fn is_google_workspace_document(&self) -> bool {
        matches!(
            self.mime_type.as_str(),
            "application/vnd.google-apps.document"
                | "application/vnd.google-apps.spreadsheet"
                | "application/vnd.google-apps.presentation"
        )
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DocsCommentsResponse {
    pub comments: Option<Vec<DocsComment>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DocsComment {
    pub id: String,
    pub content: Option<String>,
    pub author: DocsCommentAuthor,
    pub status: Option<String>,
    pub created_time: String,
    pub modified_time: String,
    pub resolved_time: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DocsCommentAuthor {
    pub email: Option<String>,
    pub name: Option<String>,
    pub picture: Option<String>,
}
