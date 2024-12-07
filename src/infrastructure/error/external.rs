use std::fmt;
use std::time::Duration;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq)]
pub enum ExternalErrorKind {
    Authentication,
    Authorization,
    ServiceUnavailable,
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

    pub fn is_retryable(&self) -> bool {
        matches!(self.kind, ExternalErrorKind::ServiceUnavailable)
    }

    pub fn error_code(&self) -> &'static str {
        match self.kind {
            ExternalErrorKind::Authentication => "EXTERNAL_AUTHENTICATION_ERROR",
            ExternalErrorKind::Authorization => "EXTERNAL_AUTHORIZATION_ERROR",
            ExternalErrorKind::ServiceUnavailable => "EXTERNAL_SERVICE_UNAVAILABLE",
            ExternalErrorKind::Other => "EXTERNAL_ERROR",
        }
    }
}
