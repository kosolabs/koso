-- Google Workspace document connections
CREATE TABLE google_workspace_connections (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_email VARCHAR(320) NOT NULL,
    google_account_id VARCHAR(255) NOT NULL,
    refresh_token TEXT NOT NULL,
    access_token TEXT,
    token_expires_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(user_email, google_account_id)
);

-- Document tracking configuration
CREATE TABLE google_workspace_documents (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id VARCHAR(36) NOT NULL,
    google_document_id VARCHAR(255) NOT NULL,
    document_type VARCHAR(20) NOT NULL, -- 'docs', 'sheets', 'slides'
    document_title TEXT NOT NULL,
    document_url TEXT NOT NULL,
    last_sync_at TIMESTAMPTZ DEFAULT NOW(),
    sync_enabled BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    FOREIGN KEY (project_id) REFERENCES projects(project_id),
    UNIQUE(project_id, google_document_id)
);

-- Comment and reviewer tracking
CREATE TABLE google_workspace_tasks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id VARCHAR(36) NOT NULL,
    google_document_id VARCHAR(255) NOT NULL,
    google_comment_id VARCHAR(255),
    google_reviewer_id VARCHAR(255),
    task_id VARCHAR(255), -- References Koso task
    task_type VARCHAR(20) NOT NULL, -- 'comment', 'reviewer'
    status VARCHAR(50) NOT NULL, -- 'pending', 'resolved', 'archived'
    assignee_email VARCHAR(320),
    due_date TIMESTAMPTZ,
    last_sync_at TIMESTAMPTZ DEFAULT NOW(),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    FOREIGN KEY (project_id) REFERENCES projects(project_id),
    FOREIGN KEY (google_document_id) REFERENCES google_workspace_documents(google_document_id)
);

-- Create indexes for performance
CREATE INDEX idx_google_workspace_connections_user ON google_workspace_connections(user_email);
CREATE INDEX idx_google_workspace_documents_project ON google_workspace_documents(project_id);
CREATE INDEX idx_google_workspace_documents_sync ON google_workspace_documents(sync_enabled, last_sync_at);
CREATE INDEX idx_google_workspace_tasks_project ON google_workspace_tasks(project_id);
CREATE INDEX idx_google_workspace_tasks_document ON google_workspace_tasks(google_document_id);
CREATE INDEX idx_google_workspace_tasks_status ON google_workspace_tasks(status);
