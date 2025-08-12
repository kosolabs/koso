# Google Workspace Integration

This plugin integrates Google Workspace (Docs, Sheets, Slides) with Koso by automatically creating and syncing tasks for comments and required reviewers.

## Features

- **Comment Tracking**: Automatically creates Koso tasks for Google Docs comments
- **Reviewer Tracking**: Tracks required reviewers and creates tasks for review requests
- **Automatic Sync**: Background service syncs documents every 15 minutes
- **OAuth Integration**: Secure authentication using Google OAuth 2.0
- **Multi-Document Support**: Connect multiple Google Workspace documents to projects

## Setup

### 1. Google Cloud Console Setup

1. Go to [Google Cloud Console](https://console.cloud.google.com/)
2. Create a new project or select an existing one
3. Enable the following APIs:
   - Google Drive API
   - Google Docs API
   - Google Sheets API
   - Google Slides API
4. Create OAuth 2.0 credentials:
   - Go to "APIs & Services" > "Credentials"
   - Click "Create Credentials" > "OAuth 2.0 Client IDs"
   - Set application type to "Web application"
   - Add authorized redirect URIs (e.g., `http://localhost:3000/plugins/google-workspace/connect`)
   - Note down the Client ID and Client Secret

### 2. Configuration

Add your Google Workspace credentials to the configuration:

```json
{
  "plugins": {
    "google_workspace": {
      "client_id": "your_google_client_id",
      "client_secret": "your_google_client_secret"
    }
  }
}
```

### 3. Secrets

Store your Google Workspace credentials as secrets:

```bash
# Create secrets directory
mkdir -p secrets/google_workspace

# Store client ID and secret
echo "your_google_client_id" > secrets/google_workspace/client_id
echo "your_google_client_secret" > secrets/google_workspace/client_secret
```

## Usage

### Connecting a Project

1. Navigate to your project settings
2. Click "Connect Google Workspace"
3. Complete the OAuth flow with Google
4. Select documents to track

### API Endpoints

#### Connect Project

```
POST /plugins/google-workspace/connect
{
  "project_id": "project_uuid",
  "authorization_code": "oauth_code",
  "redirect_uri": "http://localhost:3000/callback"
}
```

#### Discover Documents

```
GET /plugins/google-workspace/documents?project_id=project_uuid
```

#### Connect Document

```
POST /plugins/google-workspace/documents/connect
{
  "project_id": "project_uuid",
  "document_id": "google_doc_id",
  "document_type": "docs"
}
```

#### Sync Document

```
POST /plugins/google-workspace/documents/sync
{
  "project_id": "project_uuid",
  "document_id": "google_doc_id"
}
```

#### List Connected Documents

```
GET /plugins/google-workspace/documents/list?project_id=project_uuid
```

## How It Works

### 1. Authentication Flow

- User initiates OAuth flow with Google
- Google returns authorization code
- Backend exchanges code for access/refresh tokens
- Tokens are stored securely in database

### 2. Document Discovery

- Plugin queries Google Drive API for accessible documents
- Filters for Google Workspace document types (Docs, Sheets, Slides)
- Shows connection status for each document

### 3. Task Creation

- **Comments**: Creates tasks with `kind: "google_comment"`
- **Reviewers**: Creates tasks with `kind: "google_reviewer"`
- Tasks include document metadata and original URLs

### 4. Background Sync

- Service runs every 15 minutes
- Checks for new comments and reviewer changes
- Updates existing tasks or creates new ones
- Maintains sync timestamps

## Database Schema

### Tables

- `google_workspace_connections`: User OAuth connections
- `google_workspace_documents`: Document tracking configuration
- `google_workspace_tasks`: Comment/reviewer task mappings

### Task Kinds

- `google_comment`: Tasks created from Google Docs comments
- `google_reviewer`: Tasks created from required reviewers

## Configuration Options

### Per-Project Settings

```json
{
  "sync_frequency": "15m",
  "auto_create_tasks": true,
  "sync_comments": true,
  "sync_reviewers": true,
  "include_resolved": false,
  "default_assignee": "project_owner"
}
```

### Global Settings

```json
{
  "max_documents_per_project": 50,
  "max_sync_frequency": "15m",
  "webhook_enabled": true,
  "batch_sync_size": 100
}
```

## Security

- OAuth tokens are encrypted at rest
- Automatic token refresh handling
- Minimal API scope requests
- User consent required for document access

## Troubleshooting

### Common Issues

1. **OAuth Errors**: Check client ID/secret and redirect URIs
2. **Sync Failures**: Verify document permissions and API quotas
3. **Missing Comments**: Ensure Google Docs API is enabled
4. **Token Expiry**: Check refresh token logic

### Debug Logging

Enable debug logging to troubleshoot issues:

```bash
RUST_LOG=debug cargo run
```

## Future Enhancements

- [ ] Google Sheets comments support
- [ ] Google Slides comments support
- [ ] Real-time webhook notifications
- [ ] Bidirectional sync (Koso → Google)
- [ ] Advanced filtering and search
- [ ] Bulk operations
- [ ] Integration with Google Calendar deadlines
