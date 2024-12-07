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

// Error handling trait
pub trait AppResultExt<T> {
    fn with_context(self, context: impl Into<String>) -> Result<T, AppError>;
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
    pub fn is_retryable(&self) -> bool {
        match self {
            AppError::Infrastructure(e) => e.is_retryable(),
            AppError::Domain(e) => e.is_retryable(),
            AppError::External(e) => e.is_retryable(),
            AppError::Network(e) => e.is_retryable(),
            AppError::Parse(e) => e.is_retryable(),
        }
    }

    pub fn retry_after(&self) -> Option<Duration> {
        match self {
            AppError::Infrastructure(_) => None,
            AppError::Domain(_) => None,
            AppError::External(e) => e.retry_after,
            AppError::Network(e) => e.retry_after,
            AppError::Parse(_) => None,
        }
    }

    pub fn error_code(&self) -> &'static str {
        match self {
            AppError::Infrastructure(e) => e.error_code(),
            AppError::Domain(e) => e.error_code(),
            AppError::External(e) => e.error_code(),
            AppError::Network(e) => e.error_code(),
            AppError::Parse(e) => e.error_code(),
        }
    }

    pub fn context(&self) -> Option<&str> {
        match self {
            AppError::Infrastructure(e) => Some(&e.message),
            AppError::Domain(e) => e.context.as_deref().or(Some(&e.message)),
            AppError::External(e) => Some(&e.message),
            AppError::Network(e) => Some(&e.message),
            AppError::Parse(e) => Some(&e.message),
        }
    }

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
    fn from(err: DieselError) -> Self {
        AppError::Infrastructure(InfrastructureError::new(
            InfrastructureErrorKind::Database,
            err.to_string(),
            Some(Box::new(err)),
        ))
    }
}

impl From<PoolError> for AppError {
    fn from(err: PoolError) -> Self {
        AppError::Infrastructure(InfrastructureError::new(
            InfrastructureErrorKind::Database,
            err.to_string(),
            Some(Box::new(err)),
        ))
    }
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::Infrastructure(InfrastructureError::new(
            InfrastructureErrorKind::IO,
            err.to_string(),
            Some(Box::new(err)),
        ))
    }
}

impl From<reqwest::Error> for AppError {
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
