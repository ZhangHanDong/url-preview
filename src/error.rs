use thiserror::Error;
use tracing::{error, warn};

#[derive(Debug, Error)]
pub enum PreviewError {
    #[error("Failed to parse URL: {0}")]
    UrlParseError(#[from] url::ParseError),

    #[error("Failed to fetch content: {0}")]
    FetchError(String),

    #[error("Failed to extract metadata: {0}")]
    ExtractError(String),

    #[error("Cache error: {0}")]
    CacheError(String),

    #[error("Rate limit exceeded: {0}")]
    RateLimitError(String),

    #[error("Invalid content type: {0}")]
    InvalidContentType(String),

    #[error("Request timeout: {0}")]
    TimeoutError(String),

    #[error("External service error: {service} - {message}")]
    ExternalServiceError { service: String, message: String },
}

impl PreviewError {
    pub fn log(&self) {
        match self {
            PreviewError::UrlParseError(e) => {
                warn!(error = %e, "URL parsing failed");
            }
            PreviewError::FetchError(e) => {
                error!(error = %e, "Content fetch failed");
            }
            PreviewError::ExtractError(e) => {
                error!(error = %e, "Metadata extraction failed");
            }
            PreviewError::CacheError(e) => {
                warn!(error = %e, "Cache operation failed");
            }
            PreviewError::RateLimitError(e) => {
                warn!(error = %e, "Rate limit exceeded");
            }
            PreviewError::InvalidContentType(e) => {
                warn!(error = %e, "Invalid content type received");
            }
            PreviewError::TimeoutError(e) => {
                warn!(error = %e, "Request timed out");
            }
            PreviewError::ExternalServiceError { service, message } => {
                error!(
                    service = %service,
                    error = %message,
                    "External service error occurred"
                );
            }
        }
    }
}
