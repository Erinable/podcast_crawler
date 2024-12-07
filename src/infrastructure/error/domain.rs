use std::fmt;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq)]
pub enum DomainErrorKind {
    Validation,
    NotFound,
    InvalidState,
    BatchProcessing,
    Other,
}

impl fmt::Display for DomainErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Validation => write!(f, "Validation error"),
            Self::NotFound => write!(f, "Not found"),
            Self::InvalidState => write!(f, "Invalid state"),
            Self::BatchProcessing => write!(f, "Batch processing error"),
            Self::Other => write!(f, "Other domain error"),
        }
    }
}

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

    pub fn error_code(&self) -> &'static str {
        match self.kind {
            DomainErrorKind::Validation => "VALIDATION_ERROR",
            DomainErrorKind::NotFound => "NOT_FOUND",
            DomainErrorKind::InvalidState => "INVALID_STATE",
            DomainErrorKind::BatchProcessing => "BATCH_PROCESSING_ERROR",
            DomainErrorKind::Other => "DOMAIN_ERROR",
        }
    }

    pub fn is_retryable(&self) -> bool {
        matches!(self.kind, DomainErrorKind::BatchProcessing)
    }
}
