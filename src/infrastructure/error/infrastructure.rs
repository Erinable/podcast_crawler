use std::fmt;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq)]
pub enum InfrastructureErrorKind {
    Database,
    IO,
    Config,
    Cache,
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

#[derive(Error, Debug)]
#[error("Infrastructure error ({kind}): {message}")]
pub struct InfrastructureError {
    pub kind: InfrastructureErrorKind,
    pub message: String,
    #[source]
    pub source: Option<Box<dyn std::error::Error + Send + Sync>>,
}

impl InfrastructureError {
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

    pub fn error_code(&self) -> &'static str {
        match self.kind {
            InfrastructureErrorKind::Database => "DATABASE_ERROR",
            InfrastructureErrorKind::IO => "IO_ERROR",
            InfrastructureErrorKind::Config => "CONFIG_ERROR",
            InfrastructureErrorKind::Cache => "CACHE_ERROR",
            InfrastructureErrorKind::Other => "INFRASTRUCTURE_ERROR",
        }
    }

    pub fn is_retryable(&self) -> bool {
        matches!(self.kind, InfrastructureErrorKind::Database | InfrastructureErrorKind::Cache)
    }
}
