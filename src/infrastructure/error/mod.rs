//! Error handling for the podcast crawler application.
//!
//! This module provides a comprehensive error handling system that covers different types of errors:
//!
//! - Domain errors (business logic)
//! - Infrastructure errors (database, caching)
//! - Network errors (HTTP requests, timeouts)
//! - External errors (third-party services)
//! - Parse errors (data parsing and validation)
//!
//! # Error Architecture
//!
//! ```text
//! ┌─────────────────┐
//! │    AppError     │
//! ├─────────────────┤
//! │  DomainError    │
//! │ InfraError      │
//! │ NetworkError    │
//! │ ExternalError   │
//! │  ParseError     │
//! └─────────────────┘
//! ```
//!
//! # Features
//!
//! - Error context and chaining
//! - Automatic error logging
//! - Retry policies
//! - Error codes for client responses
//!
//! # Example
//!
//! ```rust
//! use podcast_crawler::infrastructure::error::{AppError, AppResultExt};
//!
//! fn example() -> Result<(), AppError> {
//!     some_operation()
//!         .with_context("Failed to perform operation")
//!         .log_error()
//! }
//! ```

pub mod domain;
pub mod external;
pub mod infrastructure;
pub mod macros;
pub mod network;
pub mod parse;

pub use self::domain::{DomainError, DomainErrorKind};
pub use self::external::{ExternalError, ExternalErrorKind};
pub use self::infrastructure::{InfrastructureError, InfrastructureErrorKind};
pub use self::network::{NetworkError, NetworkErrorKind};
pub use self::parse::{ParseError, ParseErrorKind};

use diesel::result::Error as DieselError;
use r2d2::Error as PoolError;
use std::time::Duration;
use thiserror::Error;

/// Application-wide error type that encompasses all possible errors
///
/// This enum provides a unified error type for the entire application,
/// with variants for different categories of errors.
///
/// # Variants
///
/// - `Domain` - Business logic errors
/// - `Infrastructure` - System and database errors
/// - `Network` - Network and HTTP errors
/// - `External` - Third-party service errors
/// - `Parse` - Data parsing and validation errors
#[derive(Error, Debug)]
pub enum AppError {
    #[error(transparent)]
    Infrastructure(#[from] InfrastructureError),

    #[error(transparent)]
    Domain(#[from] DomainError),

    #[error(transparent)]
    External(#[from] ExternalError),

    #[error(transparent)]
    Network(#[from] NetworkError),

    #[error(transparent)]
    Parse(#[from] ParseError),
}

pub type AppResult<T> = Result<T, AppError>;

/// Error handling trait
///
/// This trait provides additional error handling functionality for `Result` types.
///
/// # Features
///
/// - Context addition
/// - Error logging
/// - Error chaining
///
/// # Example
///
/// ```rust
/// use podcast_crawler::infrastructure::error::AppResultExt;
///
/// fn example<T, E: Into<AppError>>(result: Result<T, E>) -> Result<T, AppError> {
///     result
///         .with_context("Operation failed")
///         .log_error()
/// }
/// ```
pub trait AppResultExt<T> {
    /// Adds context to an error
    ///
    /// This method allows adding additional context information to an error
    /// without losing the original error details.
    ///
    /// # Arguments
    ///
    /// * `context` - Additional context information
    ///
    /// # Returns
    ///
    /// Returns the original result wrapped in `AppError` with added context
    fn with_context(self, context: impl Into<String>) -> Result<T, AppError>;

    /// Logs an error if present
    ///
    /// This method automatically logs any error using the application's
    /// logging system before returning it.
    ///
    /// # Returns
    ///
    /// Returns the original result after logging any error
    fn log_error(self) -> Result<T, AppError>;
}

impl<T, E> AppResultExt<T> for Result<T, E>
where
    E: Into<AppError> + Clone,
{
    fn with_context(self, context: impl Into<String>) -> Result<T, AppError> {
        self.map_err(|e| {
            let mut err: AppError = e.into();
            if let Some(context_str) = err.context() {
                err.set_context(format!("{}: {}", context.into(), context_str));
            } else {
                err.set_context(context.into());
            }
            err
        })
    }

    fn log_error(self) -> Result<T, AppError> {
        if let Err(ref e) = self {
            let err: AppError = e.clone().into();
            tracing::error!(error = %err, "Operation failed");
        }
        self.map_err(Into::into)
    }
}

impl AppError {
    /// Checks if the error is retryable
    ///
    /// Determines whether the operation that caused this error
    /// can be retried based on the error type and context.
    ///
    /// # Returns
    ///
    /// Returns `true` if the operation can be retried, `false` otherwise
    pub fn is_retryable(&self) -> bool {
        match self {
            AppError::Infrastructure(e) => e.is_retryable(),
            AppError::Domain(e) => e.is_retryable(),
            AppError::External(e) => e.is_retryable(),
            AppError::Network(e) => e.is_retryable(),
            AppError::Parse(e) => e.is_retryable(),
        }
    }

    /// Gets the recommended retry delay
    ///
    /// For retryable errors, this method returns the recommended
    /// duration to wait before retrying the operation.
    ///
    /// # Returns
    ///
    /// Returns `Some(Duration)` with the recommended delay, or `None` if not applicable
    pub fn retry_after(&self) -> Option<Duration> {
        match self {
            AppError::Infrastructure(_) => None,
            AppError::Domain(_) => None,
            AppError::External(e) => e.retry_after,
            AppError::Network(e) => e.retry_after,
            AppError::Parse(_) => None,
        }
    }

    /// Gets the error code
    ///
    /// Returns a unique error code that can be used to identify
    /// the type of error in client responses.
    ///
    /// # Returns
    ///
    /// Returns a static string containing the error code
    pub fn error_code(&self) -> &'static str {
        match self {
            AppError::Infrastructure(e) => e.error_code(),
            AppError::Domain(e) => e.error_code(),
            AppError::External(e) => e.error_code(),
            AppError::Network(e) => e.error_code(),
            AppError::Parse(e) => e.error_code(),
        }
    }

    /// Gets the error context
    ///
    /// Returns any additional context information that was added
    /// to the error.
    ///
    /// # Returns
    ///
    /// Returns `Some(&str)` with the context message, or `None` if no context was set
    pub fn context(&self) -> Option<&str> {
        match self {
            AppError::Infrastructure(e) => Some(&e.message),
            AppError::Domain(e) => e.context.as_deref().or(Some(&e.message)),
            AppError::External(e) => Some(&e.message),
            AppError::Network(e) => Some(&e.message),
            AppError::Parse(e) => Some(&e.message),
        }
    }

    /// Sets the error context
    ///
    /// Adds or updates the context information for this error.
    ///
    /// # Arguments
    ///
    /// * `context` - The context message to set
    pub fn set_context(&mut self, context: String) {
        match self {
            AppError::Infrastructure(e) => e.message = context,
            AppError::Domain(e) => {
                if e.context.is_some() {
                    e.message = format!("{}: {}", context, e.message);
                } else {
                    e.context = Some(context);
                }
            }
            AppError::External(e) => e.message = context,
            AppError::Network(e) => e.message = context,
            AppError::Parse(e) => e.message = context,
        }
    }
}

// Error conversions
impl From<DieselError> for AppError {
    /// Converts a Diesel error into an AppError
    ///
    /// Maps database-specific errors to appropriate AppError variants
    fn from(err: DieselError) -> Self {
        AppError::Infrastructure(InfrastructureError::new(
            InfrastructureErrorKind::Database,
            err.to_string(),
            Some(Box::new(err)),
        ))
    }
}

impl From<PoolError> for AppError {
    /// Converts a connection pool error into an AppError
    ///
    /// Maps connection pool errors to infrastructure errors
    fn from(err: PoolError) -> Self {
        AppError::Infrastructure(InfrastructureError::new(
            InfrastructureErrorKind::Database,
            err.to_string(),
            Some(Box::new(err)),
        ))
    }
}

impl From<std::io::Error> for AppError {
    /// Converts an IO error into an AppError
    ///
    /// Maps system IO errors to infrastructure errors
    fn from(err: std::io::Error) -> Self {
        AppError::Infrastructure(InfrastructureError::new(
            InfrastructureErrorKind::IO,
            err.to_string(),
            Some(Box::new(err)),
        ))
    }
}

impl From<reqwest::Error> for AppError {
    /// Converts a reqwest error into an AppError
    ///
    /// Maps HTTP client errors to appropriate network or external errors
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            AppError::Network(NetworkError::new(
                NetworkErrorKind::Timeout,
                err.to_string(),
                None,
                Some(Box::new(err)),
            ))
        } else if err.is_connect() {
            AppError::Network(NetworkError::new(
                NetworkErrorKind::Connection,
                err.to_string(),
                None,
                Some(Box::new(err)),
            ))
        } else {
            AppError::External(ExternalError::new(
                ExternalErrorKind::ServiceUnavailable,
                err.to_string(),
                None,
                Some(Box::new(err)),
            ))
        }
    }
}
