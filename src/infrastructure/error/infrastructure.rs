//! Infrastructure-related error types and handling.
//!
//! This module provides error types for infrastructure-level operations including:
//! - Database operations
//! - File I/O
//! - Configuration
//! - Caching
//!
//! # Error Structure
//!
//! ```text
//! ┌─────────────────────────┐
//! │  InfrastructureError    │
//! ├─────────────────────────┤
//! │ - kind: ErrorKind       │
//! │ - message: String       │
//! │ - source: Option<Error> │
//! └─────────────────────────┘
//! ```
//!
//! # Example
//!
//! ```rust
//! use podcast_crawler::infrastructure::error::infrastructure::{
//!     InfrastructureError,
//!     InfrastructureErrorKind,
//! };
//!
//! fn example() -> Result<(), InfrastructureError> {
//!     Err(InfrastructureError::new(
//!         InfrastructureErrorKind::Database,
//!         "Failed to connect to database",
//!         None,
//!     ))
//! }
//! ```

use std::fmt;
use thiserror::Error;

/// Types of infrastructure errors
///
/// This enum represents different categories of infrastructure-level errors
/// that can occur in the application.
#[derive(Debug, Clone, PartialEq)]
pub enum InfrastructureErrorKind {
    /// Database-related errors (connection, query, etc.)
    Database,
    /// File system and I/O errors
    IO,
    /// Configuration loading and validation errors
    Config,
    /// Cache operation errors
    Cache,
    /// Other infrastructure errors
    Other,
}

impl fmt::Display for InfrastructureErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Database => write!(f, "Database error"),
            Self::IO => write!(f, "IO error"),
            Self::Config => write!(f, "Configuration error"),
            Self::Cache => write!(f, "Cache error"),
            Self::Other => write!(f, "Other infrastructure error"),
        }
    }
}

/// Infrastructure error type
///
/// Represents errors that occur in the infrastructure layer of the application,
/// such as database errors, I/O errors, or configuration errors.
///
/// # Fields
///
/// * `kind` - The type of infrastructure error
/// * `message` - A human-readable error message
/// * `source` - The underlying error that caused this error, if any
///
/// # Example
///
/// ```rust
/// use podcast_crawler::infrastructure::error::infrastructure::{
///     InfrastructureError,
///     InfrastructureErrorKind,
/// };
///
/// let error = InfrastructureError::new(
///     InfrastructureErrorKind::Database,
///     "Failed to execute query",
///     None,
/// );
/// assert_eq!(error.error_code(), "DATABASE_ERROR");
/// ```
#[derive(Error, Debug)]
#[error("Infrastructure error ({kind}): {message}")]
pub struct InfrastructureError {
    pub kind: InfrastructureErrorKind,
    pub message: String,
    #[source]
    pub source: Option<Box<dyn std::error::Error + Send + Sync>>,
}

impl InfrastructureError {
    /// Creates a new infrastructure error
    ///
    /// # Arguments
    ///
    /// * `kind` - The type of infrastructure error
    /// * `message` - A human-readable error message
    /// * `source` - The underlying error that caused this error, if any
    ///
    /// # Returns
    ///
    /// Returns a new `InfrastructureError` instance
    pub fn new(
        kind: InfrastructureErrorKind,
        message: impl Into<String>,
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    ) -> Self {
        Self {
            kind,
            message: message.into(),
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
    /// - `DATABASE_ERROR` for database errors
    /// - `IO_ERROR` for I/O errors
    /// - `CONFIG_ERROR` for configuration errors
    /// - `CACHE_ERROR` for cache errors
    /// - `INFRASTRUCTURE_ERROR` for other errors
    pub fn error_code(&self) -> &'static str {
        match self.kind {
            InfrastructureErrorKind::Database => "DATABASE_ERROR",
            InfrastructureErrorKind::IO => "IO_ERROR",
            InfrastructureErrorKind::Config => "CONFIG_ERROR",
            InfrastructureErrorKind::Cache => "CACHE_ERROR",
            InfrastructureErrorKind::Other => "INFRASTRUCTURE_ERROR",
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
    /// Currently, only certain database errors are considered retryable.
    pub fn is_retryable(&self) -> bool {
        matches!(
            self.kind,
            InfrastructureErrorKind::Database | InfrastructureErrorKind::Cache
        )
    }
}
