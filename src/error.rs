use thiserror::Error;
#[cfg(feature = "logging")]
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

    #[error("DNS resolution failed: {0}")]
    DnsError(String),

    #[error("Connection error: {0}")]
    ConnectionError(String),

    #[error("HTTP {status}: {message}")]
    HttpError { status: u16, message: String },

    #[error("Server error (5xx): {status} - {message}")]
    ServerError { status: u16, message: String },

    #[error("Client error (4xx): {status} - {message}")]
    ClientError { status: u16, message: String },

    #[error("External service error: {service} - {message}")]
    ExternalServiceError { service: String, message: String },

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Concurrency limit reached")]
    ConcurrencyLimitError,

    #[error("Resource not found: {0}")]
    NotFound(String),
}

impl PreviewError {
    pub fn log(&self) {
        #[cfg(feature = "logging")]
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
            PreviewError::ParseError(e) => {
                error!(error = %e, "Parse error occurred");
            }
            PreviewError::ConcurrencyLimitError => {
                warn!("Concurrency limit reached");
            }
            PreviewError::NotFound(e) => {
                warn!(error = %e, "Resource not found");
            }
            PreviewError::DnsError(e) => {
                error!(error = %e, "DNS resolution failed");
            }
            PreviewError::ConnectionError(e) => {
                error!(error = %e, "Connection failed");
            }
            PreviewError::HttpError { status, message } => {
                warn!(status = %status, error = %message, "HTTP error");
            }
            PreviewError::ServerError { status, message } => {
                error!(status = %status, error = %message, "Server error");
            }
            PreviewError::ClientError { status, message } => {
                warn!(status = %status, error = %message, "Client error");
            }
        }
        #[cfg(not(feature = "logging"))]
        {
            // No-op when logging is disabled
        }
    }

    /// Convert a reqwest error into a more specific PreviewError
    pub fn from_reqwest_error(error: reqwest::Error) -> Self {
        if error.is_timeout() {
            PreviewError::TimeoutError(error.to_string())
        } else if error.is_connect() {
            // Connection errors including DNS
            let error_msg = error.to_string();
            if error_msg.contains("dns")
                || error_msg.contains("resolve")
                || error_msg.contains("lookup")
            {
                PreviewError::DnsError(error_msg)
            } else {
                PreviewError::ConnectionError(error_msg)
            }
        } else if let Some(status) = error.status() {
            let status_code = status.as_u16();
            let message = error.to_string();

            match status_code {
                404 => PreviewError::NotFound(message),
                400..=499 => PreviewError::ClientError {
                    status: status_code,
                    message,
                },
                500..=599 => PreviewError::ServerError {
                    status: status_code,
                    message,
                },
                _ => PreviewError::HttpError {
                    status: status_code,
                    message,
                },
            }
        } else {
            // Generic fetch error for other cases
            PreviewError::FetchError(error.to_string())
        }
    }
}
