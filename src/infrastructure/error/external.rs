//! External service error types and handling.
//!
//! This module provides error types for external service operations including:
//! - Authentication and authorization
//! - Service availability
//! - Third-party API interactions
//! - Rate limiting and retry handling
//!
//! # Error Structure
//!
//! ```text
//! ┌─────────────────────────┐
//! │     ExternalError       │
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
//! use podcast_crawler::infrastructure::error::external::{ExternalError, ExternalErrorKind};
//! use std::time::Duration;
//!
//! async fn call_external_api() -> Result<(), ExternalError> {
//!     if !is_authenticated() {
//!         return Err(ExternalError::new(
//!             ExternalErrorKind::Authentication,
//!             "Invalid API key",
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

/// Types of external service errors
///
/// This enum represents different categories of errors that can occur
/// when interacting with external services and APIs.
#[derive(Debug, Clone, PartialEq)]
pub enum ExternalErrorKind {
    /// Authentication failures (e.g., invalid credentials)
    Authentication,
    /// Authorization failures (e.g., insufficient permissions)
    Authorization,
    /// Service unavailability (e.g., maintenance, outage)
    ServiceUnavailable,
    /// Other external service errors
    Other,
}

impl fmt::Display for ExternalErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Authentication => write!(f, "Authentication error"),
            Self::Authorization => write!(f, "Authorization error"),
            Self::ServiceUnavailable => write!(f, "Service unavailable"),
            Self::Other => write!(f, "Other external error"),
        }
    }
}

/// External error type
///
/// Represents errors that occur during interactions with external services,
/// such as authentication failures, service outages, or API errors.
///
/// # Fields
///
/// * `kind` - The type of external error
/// * `message` - A human-readable error message
/// * `retry_after` - Optional duration to wait before retry
/// * `source` - The underlying error that caused this error, if any
///
/// # Example
///
/// ```rust
/// use podcast_crawler::infrastructure::error::external::{ExternalError, ExternalErrorKind};
/// use std::time::Duration;
///
/// let error = ExternalError::new(
///     ExternalErrorKind::ServiceUnavailable,
///     "Service is under maintenance",
///     Some(Duration::from_secs(300)),
///     None,
/// );
/// assert_eq!(error.error_code(), "EXTERNAL_SERVICE_UNAVAILABLE");
/// assert!(error.is_retryable());
/// ```
#[derive(Error, Debug)]
#[error("External error ({kind}): {message}")]
pub struct ExternalError {
    pub kind: ExternalErrorKind,
    pub message: String,
    pub retry_after: Option<Duration>,
    #[source]
    pub source: Option<Box<dyn std::error::Error + Send + Sync>>,
}

impl ExternalError {
    /// Creates a new external error
    ///
    /// # Arguments
    ///
    /// * `kind` - The type of external error
    /// * `message` - A human-readable error message
    /// * `retry_after` - Optional duration to wait before retry
    /// * `source` - The underlying error that caused this error (optional)
    ///
    /// # Returns
    ///
    /// Returns a new `ExternalError` instance
    ///
    /// # Example
    ///
    /// ```rust
    /// use podcast_crawler::infrastructure::error::external::{ExternalError, ExternalErrorKind};
    /// use std::time::Duration;
    ///
    /// let error = ExternalError::new(
    ///     ExternalErrorKind::Authorization,
    ///     "Insufficient permissions",
    ///     None,
    ///     None,
    /// );
    /// ```
    pub fn new(
        kind: ExternalErrorKind,
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
    /// Determines whether the external service operation that caused this error
    /// can be retried.
    ///
    /// # Returns
    ///
    /// Returns `true` if the operation can be retried, `false` otherwise.
    /// Currently, only service unavailability errors are considered retryable
    /// since they are typically temporary issues.
    pub fn is_retryable(&self) -> bool {
        matches!(self.kind, ExternalErrorKind::ServiceUnavailable)
    }

    /// Gets the error code for this error
    ///
    /// Returns a unique code that identifies this type of error.
    /// These codes can be used in API responses or logging.
    ///
    /// # Returns
    ///
    /// Returns a static string containing the error code:
    /// - `EXTERNAL_AUTHENTICATION_ERROR` for authentication failures
    /// - `EXTERNAL_AUTHORIZATION_ERROR` for authorization failures
    /// - `EXTERNAL_SERVICE_UNAVAILABLE` for service unavailability
    /// - `EXTERNAL_ERROR` for other external service errors
    pub fn error_code(&self) -> &'static str {
        match self.kind {
            ExternalErrorKind::Authentication => "EXTERNAL_AUTHENTICATION_ERROR",
            ExternalErrorKind::Authorization => "EXTERNAL_AUTHORIZATION_ERROR",
            ExternalErrorKind::ServiceUnavailable => "EXTERNAL_SERVICE_UNAVAILABLE",
            ExternalErrorKind::Other => "EXTERNAL_ERROR",
        }
    }
}
