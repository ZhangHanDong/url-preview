use url_preview::{PreviewError, PreviewService};

#[tokio::test]
async fn test_dns_error() {
    let service = PreviewService::new();

    // Test with a non-existent domain
    let result = service
        .generate_preview("https://this-domain-definitely-does-not-exist-12345.com")
        .await;

    assert!(result.is_err());
    match result.unwrap_err() {
        PreviewError::DnsError(msg) => {
            println!("DNS Error detected: {}", msg);
            assert!(msg.to_lowercase().contains("dns") || msg.to_lowercase().contains("resolve"));
        }
        PreviewError::ConnectionError(msg) => {
            // Also acceptable if DNS error is reported as connection error
            println!("Connection Error detected: {}", msg);
            // Connection errors can have various messages
        }
        e => panic!("Expected DnsError or ConnectionError, got: {:?}", e),
    }
}

#[tokio::test]
async fn test_http_404_error() {
    let service = PreviewService::new();

    // httpbin.org provides reliable test endpoints
    let result = service
        .generate_preview("https://httpbin.org/status/404")
        .await;

    assert!(result.is_err());
    match result.unwrap_err() {
        PreviewError::NotFound(msg) => {
            println!("404 Not Found detected: {}", msg);
            assert!(msg.contains("not found"));
        }
        e => panic!("Expected NotFound error, got: {:?}", e),
    }
}

#[tokio::test]
async fn test_http_500_error() {
    let service = PreviewService::new();

    // Test with a 500 error
    let result = service
        .generate_preview("https://httpbin.org/status/500")
        .await;

    assert!(result.is_err());
    match result.unwrap_err() {
        PreviewError::ServerError { status, message } => {
            println!("Server Error detected: {} - {}", status, message);
            assert_eq!(status, 500);
        }
        e => panic!("Expected ServerError, got: {:?}", e),
    }
}

#[tokio::test]
async fn test_http_400_error() {
    let service = PreviewService::new();

    // Test with a 400 error
    let result = service
        .generate_preview("https://httpbin.org/status/400")
        .await;

    assert!(result.is_err());
    match result.unwrap_err() {
        PreviewError::ClientError { status, message } => {
            println!("Client Error detected: {} - {}", status, message);
            assert_eq!(status, 400);
        }
        e => panic!("Expected ClientError, got: {:?}", e),
    }
}

#[tokio::test]
async fn test_timeout_error() {
    let service = PreviewService::new();

    // httpbin.org/delay/{seconds} endpoint that delays response
    // Using a very long delay to trigger timeout
    let result = service
        .generate_preview("https://httpbin.org/delay/60")
        .await;

    assert!(result.is_err());
    let error = result.unwrap_err();

    // Timeout might be reported as TimeoutError or generic FetchError
    match &error {
        PreviewError::TimeoutError(msg) => {
            println!("Timeout Error detected: {}", msg);
            // Just verify we got a timeout error, message format varies
        }
        PreviewError::FetchError(msg) => {
            // Also acceptable if timeout is reported as generic fetch error
            println!("Fetch Error (possibly timeout): {}", msg);
        }
        e => panic!("Expected TimeoutError or FetchError, got: {:?}", e),
    }
}

#[tokio::test]
#[cfg(feature = "github")]
async fn test_github_api_error_details() {
    let service = PreviewService::new();

    // Test with a private/non-existent repo
    let result = service
        .generate_preview("https://github.com/private-org-12345/private-repo-12345")
        .await;

    assert!(result.is_err());
    let error = result.unwrap_err();

    // Should get a specific error, not just generic FetchError
    match error {
        PreviewError::NotFound(msg) => {
            println!("GitHub repo not found: {}", msg);
            assert!(msg.contains("not found"));
        }
        PreviewError::ClientError { status, message } => {
            println!("GitHub client error: {} - {}", status, message);
            assert!(status >= 400 && status < 500);
        }
        e => {
            println!("Error type: {:?}", e);
            // At least it should not be a generic FetchError
            if let PreviewError::FetchError(_) = e {
                panic!("Should not get generic FetchError for GitHub 404");
            }
        }
    }
}
