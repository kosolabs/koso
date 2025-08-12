-- Drop indexes
DROP INDEX IF EXISTS idx_google_workspace_tasks_status;
DROP INDEX IF EXISTS idx_google_workspace_tasks_document;
DROP INDEX IF EXISTS idx_google_workspace_tasks_project;
DROP INDEX IF EXISTS idx_google_workspace_documents_sync;
DROP INDEX IF EXISTS idx_google_workspace_documents_project;
DROP INDEX IF EXISTS idx_google_workspace_connections_user;

-- Drop tables
DROP TABLE IF EXISTS google_workspace_tasks;
DROP TABLE IF EXISTS google_workspace_documents;
DROP TABLE IF EXISTS google_workspace_connections;
