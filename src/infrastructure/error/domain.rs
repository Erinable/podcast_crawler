//! Domain-specific error types and handling.
//!
//! This module provides error types for domain-level operations including:
//! - Data validation
//! - Entity state management
//! - Business rule enforcement
//! - Batch processing operations
//!
//! # Error Structure
//!
//! ```text
//! ┌─────────────────────────┐
//! │     DomainError         │
//! ├─────────────────────────┤
//! │ - kind: ErrorKind       │
//! │ - message: String       │
//! │ - context: Option<Str>  │
//! │ - source: Option<Error> │
//! └─────────────────────────┘
//! ```
//!
//! # Example
//!
//! ```rust
//! use podcast_crawler::infrastructure::error::domain::{DomainError, DomainErrorKind};
//!
//! fn validate_podcast(title: &str) -> Result<(), DomainError> {
//!     if title.is_empty() {
//!         return Err(DomainError::new(
//!             DomainErrorKind::Validation,
//!             "Podcast title cannot be empty",
//!             None,
//!             None,
//!         ));
//!     }
//!     Ok(())
//! }
//! ```

use std::fmt;
use thiserror::Error;

/// Types of domain errors
///
/// This enum represents different categories of domain-level errors
/// that can occur in the application's business logic.
#[derive(Debug, Clone, PartialEq)]
pub enum DomainErrorKind {
    /// Data validation errors
    Validation,
    /// Entity not found errors
    NotFound,
    /// Invalid entity state errors
    InvalidState,
    /// Batch processing errors
    BatchProcessing,
    /// Other domain-specific errors
    Other,
    /// Unexpected errors
    Unexpected,
}

impl fmt::Display for DomainErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Validation => write!(f, "Validation error"),
            Self::NotFound => write!(f, "Not found"),
            Self::InvalidState => write!(f, "Invalid state"),
            Self::BatchProcessing => write!(f, "Batch processing error"),
            Self::Other => write!(f, "Other domain error"),
            Self::Unexpected => write!(f, "Unexpected error"),
        }
    }
}

/// Domain error type
///
/// Represents errors that occur in the domain layer of the application,
/// such as validation errors, business rule violations, or invalid state transitions.
///
/// # Fields
///
/// * `kind` - The type of domain error
/// * `message` - A human-readable error message
/// * `context` - Additional context about the error
/// * `source` - The underlying error that caused this error, if any
///
/// # Example
///
/// ```rust
/// use podcast_crawler::infrastructure::error::domain::{DomainError, DomainErrorKind};
///
/// let error = DomainError::new(
///     DomainErrorKind::Validation,
///     "Invalid podcast title",
///     Some("Title length must be between 1 and 100 characters".to_string()),
///     None,
/// );
/// assert_eq!(error.error_code(), "VALIDATION_ERROR");
/// ```
#[derive(Error, Debug)]
#[error("Domain error ({kind}): {message}")]
pub struct DomainError {
    pub kind: DomainErrorKind,
    pub message: String,
    pub context: Option<String>,
    #[source]
    pub source: Option<Box<dyn std::error::Error + Send + Sync>>,
}

impl DomainError {
    /// Creates a new domain error
    ///
    /// # Arguments
    ///
    /// * `kind` - The type of domain error
    /// * `message` - A human-readable error message
    /// * `context` - Additional context about the error (optional)
    /// * `source` - The underlying error that caused this error (optional)
    ///
    /// # Returns
    ///
    /// Returns a new `DomainError` instance
    ///
    /// # Example
    ///
    /// ```rust
    /// use podcast_crawler::infrastructure::error::domain::{DomainError, DomainErrorKind};
    ///
    /// let error = DomainError::new(
    ///     DomainErrorKind::NotFound,
    ///     "Podcast not found",
    ///     Some("ID: 12345".to_string()),
    ///     None,
    /// );
    /// ```
    pub fn new(
        kind: DomainErrorKind,
        message: impl Into<String>,
        context: Option<String>,
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    ) -> Self {
        Self {
            kind,
            message: message.into(),
            context,
            source,
        }
    }

    /// Gets the error code for this error
    ///
    /// Returns a unique code that identifies this type of error.
    /// These codes can be used in API responses or logging.
    ///
    /// # Returns
    ///
    /// Returns a static string containing the error code:
    /// - `VALIDATION_ERROR` for validation errors
    /// - `NOT_FOUND_ERROR` for not found errors
    /// - `INVALID_STATE_ERROR` for invalid state errors
    /// - `BATCH_ERROR` for batch processing errors
    /// - `DOMAIN_ERROR` for other domain errors
    /// - `UNEXPECTED_ERROR` for unexpected errors
    pub fn error_code(&self) -> &'static str {
        match self.kind {
            DomainErrorKind::Validation => "VALIDATION_ERROR",
            DomainErrorKind::NotFound => "NOT_FOUND_ERROR",
            DomainErrorKind::InvalidState => "INVALID_STATE_ERROR",
            DomainErrorKind::BatchProcessing => "BATCH_ERROR",
            DomainErrorKind::Other => "DOMAIN_ERROR",
            DomainErrorKind::Unexpected => "UNEXPECTED_ERROR",
        }
    }

    /// Checks if this error is retryable
    ///
    /// Determines whether the operation that caused this error
    /// can be retried.
    ///
    /// # Returns
    ///
    /// Returns `true` if the operation can be retried, `false` otherwise.
    /// Currently, only batch processing errors are considered retryable.
    pub fn is_retryable(&self) -> bool {
        matches!(self.kind, DomainErrorKind::BatchProcessing)
    }
}
