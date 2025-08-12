#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugins::google_workspace::models::*;

    #[test]
    fn test_google_comment_is_resolved() {
        let comment = GoogleComment {
            id: "comment1".to_string(),
            content: "Test comment".to_string(),
            author: GoogleUser {
                email: "test@example.com".to_string(),
                name: "Test User".to_string(),
                picture: None,
            },
            status: "resolved".to_string(),
            created_time: chrono::Utc::now(),
            modified_time: chrono::Utc::now(),
            resolved_time: Some(chrono::Utc::now()),
            document_id: "doc1".to_string(),
            document_type: "docs".to_string(),
            document_url: "https://docs.google.com/doc1".to_string(),
        };

        assert!(comment.is_resolved());
    }

    #[test]
    fn test_google_comment_summary() {
        let long_content = "A".repeat(150);
        let comment = GoogleComment {
            id: "comment1".to_string(),
            content: long_content.clone(),
            author: GoogleUser {
                email: "test@example.com".to_string(),
                name: "Test User".to_string(),
                picture: None,
            },
            status: "open".to_string(),
            created_time: chrono::Utc::now(),
            modified_time: chrono::Utc::now(),
            resolved_time: None,
            document_id: "doc1".to_string(),
            document_type: "docs".to_string(),
            document_url: "https://docs.google.com/doc1".to_string(),
        };

        let summary = comment.summary();
        assert_eq!(summary.len(), 100);
        assert!(summary.ends_with("..."));
    }

    #[test]
    fn test_google_reviewer_status() {
        let reviewer = GoogleReviewer {
            id: "reviewer1".to_string(),
            email: "reviewer@example.com".to_string(),
            name: "Reviewer User".to_string(),
            status: "pending".to_string(),
            requested_time: chrono::Utc::now(),
            responded_time: None,
            document_id: "doc1".to_string(),
            document_type: "docs".to_string(),
            document_title: "Test Document".to_string(),
            document_url: "https://docs.google.com/doc1".to_string(),
            due_date: None,
            requestor_email: "requestor@example.com".to_string(),
        };

        assert!(reviewer.is_pending());
        assert!(!reviewer.is_approved());
        assert!(!reviewer.is_rejected());
    }

    #[test]
    fn test_google_document_type() {
        let docs_doc = GoogleDocument {
            id: "doc1".to_string(),
            name: "Test Doc".to_string(),
            mime_type: "application/vnd.google-apps.document".to_string(),
            web_view_link: "https://docs.google.com/doc1".to_string(),
            created_time: chrono::Utc::now(),
            modified_time: chrono::Utc::now(),
            owners: Vec::new(),
            writers: Vec::new(),
            readers: Vec::new(),
        };

        let sheets_doc = GoogleDocument {
            id: "sheet1".to_string(),
            name: "Test Sheet".to_string(),
            mime_type: "application/vnd.google-apps.spreadsheet".to_string(),
            web_view_link: "https://sheets.google.com/sheet1".to_string(),
            created_time: chrono::Utc::now(),
            modified_time: chrono::Utc::now(),
            owners: Vec::new(),
            writers: Vec::new(),
            readers: Vec::new(),
        };

        assert_eq!(docs_doc.document_type(), "docs");
        assert_eq!(sheets_doc.document_type(), "sheets");
        assert!(docs_doc.is_google_workspace_document());
        assert!(sheets_doc.is_google_workspace_document());
    }

    #[test]
    fn test_kind_validation() {
        let valid_kind = Kind::new("test", "Test Kind");
        assert!(valid_kind.validate().is_ok());

        let invalid_kind = Kind::new("", "Test Kind");
        assert!(invalid_kind.validate().is_err());

        let invalid_kind2 = Kind::new("test", "");
        assert!(invalid_kind2.validate().is_err());
    }

    #[test]
    fn test_nested_kind() {
        let parent = Kind::new("parent", "Parent");
        let nested = Kind::new_nested(parent, "child", "Child");
        
        assert_eq!(nested.id, "parent_child");
        assert_eq!(nested.name, "Child");
    }
}
