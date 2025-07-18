use url_preview::{PreviewError, PreviewService};

#[tokio::test]
async fn test_github_repo_not_found() {
    let service = PreviewService::new();

    // Test with a non-existent GitHub repository
    let result = service
        .generate_preview("https://github.com/nonexistent-user/nonexistent-repo")
        .await;

    assert!(result.is_err());
    match result.unwrap_err() {
        PreviewError::NotFound(msg) => {
            assert!(msg.contains("not found"));
            assert!(msg.contains("nonexistent-user/nonexistent-repo"));
        }
        _ => panic!("Expected NotFound error"),
    }
}

#[tokio::test]
async fn test_github_file_not_found() {
    let service = PreviewService::new();

    // Test with a non-existent file in a real repository
    let result = service
        .generate_preview("https://github.com/rust-lang/rust/blob/master/nonexistent-file.rs")
        .await;

    // This should return an error because the full URL (including the file path) returns 404
    assert!(result.is_err());
    match result.unwrap_err() {
        PreviewError::NotFound(_) => {}
        PreviewError::FetchError(_) => {} // Also acceptable
        _ => panic!("Expected NotFound or FetchError"),
    }
}

#[cfg(feature = "twitter")]
#[tokio::test]
async fn test_twitter_post_not_found() {
    let service = PreviewService::new();

    // Test with a non-existent tweet ID
    let result = service
        .generate_preview("https://twitter.com/twitter/status/99999999999999999999")
        .await;

    assert!(result.is_err());
    match result.unwrap_err() {
        PreviewError::NotFound(msg) => {
            assert!(msg.contains("not found"));
        }
        PreviewError::ExternalServiceError { service, .. } => {
            assert_eq!(service, "Twitter");
        }
        _ => panic!("Expected NotFound or ExternalServiceError"),
    }
}

#[tokio::test]
async fn test_valid_github_repo() {
    let service = PreviewService::new();

    // Test with a valid GitHub repository
    let result = service
        .generate_preview("https://github.com/rust-lang/rust")
        .await;

    assert!(result.is_ok());
    let preview = result.unwrap();
    assert!(preview.title.is_some());
    assert!(preview.description.is_some());
}
