//! Network-related error types and handling.
//!
//! This module provides error types for network operations including:
//! - HTTP requests and responses
//! - Connection management
//! - Rate limiting
//! - Timeouts and retries
//!
//! # Error Structure
//!
//! ```text
//! ┌─────────────────────────┐
//! │     NetworkError        │
//! ├─────────────────────────┤
//! │ - kind: ErrorKind       │
//! │ - message: String       │
//! │ - retry_after: Duration │
//! │ - source: Option<Error> │
//! └─────────────────────────┘
//! ```
//!
//! # Example
//!
//! ```rust
//! use podcast_crawler::infrastructure::error::network::{NetworkError, NetworkErrorKind};
//! use std::time::Duration;
//!
//! async fn fetch_podcast(url: &str) -> Result<(), NetworkError> {
//!     if url.is_empty() {
//!         return Err(NetworkError::new(
//!             NetworkErrorKind::InvalidResponse,
//!             "Invalid URL",
//!             None,
//!             None,
//!         ));
//!     }
//!     Ok(())
//! }
//! ```

use std::fmt;
use std::time::Duration;
use thiserror::Error;

/// Types of network errors
///
/// This enum represents different categories of network-level errors
/// that can occur during HTTP requests and other network operations.
#[derive(Debug, Clone, PartialEq)]
pub enum NetworkErrorKind {
    /// Connection establishment errors
    Connection,
    /// Request timeout errors
    Timeout,
    /// Too many redirects in request chain
    TooManyRedirects,
    /// Invalid or malformed response
    InvalidResponse,
    /// Rate limit exceeded
    RateLimit,
    /// Other network-related errors
    Other,
}

impl fmt::Display for NetworkErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Connection => write!(f, "Connection error"),
            Self::Timeout => write!(f, "Timeout"),
            Self::TooManyRedirects => write!(f, "Too many redirects"),
            Self::InvalidResponse => write!(f, "Invalid response"),
            Self::RateLimit => write!(f, "Rate limit exceeded"),
            Self::Other => write!(f, "Other network error"),
        }
    }
}

/// Network error type
///
/// Represents errors that occur during network operations,
/// such as HTTP requests, connection issues, or rate limiting.
///
/// # Fields
///
/// * `kind` - The type of network error
/// * `message` - A human-readable error message
/// * `retry_after` - Optional duration to wait before retry
/// * `source` - The underlying error that caused this error, if any
///
/// # Example
///
/// ```rust
/// use podcast_crawler::infrastructure::error::network::{NetworkError, NetworkErrorKind};
/// use std::time::Duration;
///
/// let error = NetworkError::new(
///     NetworkErrorKind::RateLimit,
///     "Rate limit exceeded",
///     Some(Duration::from_secs(60)),
///     None,
/// );
/// assert_eq!(error.error_code(), "RATE_LIMIT_ERROR");
/// assert!(error.is_retryable());
/// ```
#[derive(Error, Debug)]
#[error("Network error ({kind}): {message}")]
pub struct NetworkError {
    pub kind: NetworkErrorKind,
    pub message: String,
    pub retry_after: Option<Duration>,
    #[source]
    pub source: Option<Box<dyn std::error::Error + Send + Sync>>,
}

impl NetworkError {
    /// Creates a new network error
    ///
    /// # Arguments
    ///
    /// * `kind` - The type of network error
    /// * `message` - A human-readable error message
    /// * `retry_after` - Optional duration to wait before retry
    /// * `source` - The underlying error that caused this error (optional)
    ///
    /// # Returns
    ///
    /// Returns a new `NetworkError` instance
    ///
    /// # Example
    ///
    /// ```rust
    /// use podcast_crawler::infrastructure::error::network::{NetworkError, NetworkErrorKind};
    /// use std::time::Duration;
    ///
    /// let error = NetworkError::new(
    ///     NetworkErrorKind::Timeout,
    ///     "Request timed out",
    ///     Some(Duration::from_secs(5)),
    ///     None,
    /// );
    /// ```
    pub fn new(
        kind: NetworkErrorKind,
        message: impl Into<String>,
        retry_after: Option<Duration>,
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    ) -> Self {
        Self {
            kind,
            message: message.into(),
            retry_after,
            source,
        }
    }

    /// Checks if this error is retryable
    ///
    /// Determines whether the network operation that caused this error
    /// can be retried.
    ///
    /// # Returns
    ///
    /// Returns `true` if the operation can be retried, `false` otherwise.
    /// The following errors are considered retryable:
    /// - Connection errors
    /// - Timeout errors
    /// - Rate limit errors
    pub fn is_retryable(&self) -> bool {
        matches!(
            self.kind,
            NetworkErrorKind::Connection | NetworkErrorKind::Timeout | NetworkErrorKind::RateLimit
        )
    }

    /// Gets the error code for this error
    ///
    /// Returns a unique code that identifies this type of error.
    /// These codes can be used in API responses or logging.
    ///
    /// # Returns
    ///
    /// Returns a static string containing the error code:
    /// - `CONNECTION_ERROR` for connection errors
    /// - `TIMEOUT_ERROR` for timeout errors
    /// - `REDIRECT_ERROR` for too many redirects
    /// - `RESPONSE_ERROR` for invalid responses
    /// - `RATE_LIMIT_ERROR` for rate limit errors
    /// - `NETWORK_ERROR` for other network errors
    pub fn error_code(&self) -> &'static str {
        match self.kind {
            NetworkErrorKind::Connection => "CONNECTION_ERROR",
            NetworkErrorKind::Timeout => "TIMEOUT_ERROR",
            NetworkErrorKind::TooManyRedirects => "REDIRECT_ERROR",
            NetworkErrorKind::InvalidResponse => "RESPONSE_ERROR",
            NetworkErrorKind::RateLimit => "RATE_LIMIT_ERROR",
            NetworkErrorKind::Other => "NETWORK_ERROR",
        }
    }
}
