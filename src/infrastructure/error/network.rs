use std::fmt;
use std::time::Duration;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq)]
pub enum NetworkErrorKind {
    Connection,
    Timeout,
    TooManyRedirects,
    InvalidResponse,
    RateLimit,
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

    pub fn is_retryable(&self) -> bool {
        matches!(
            self.kind,
            NetworkErrorKind::Connection | NetworkErrorKind::Timeout | NetworkErrorKind::RateLimit
        )
    }

    pub fn error_code(&self) -> &'static str {
        match self.kind {
            NetworkErrorKind::Connection => "CONNECTION_ERROR",
            NetworkErrorKind::Timeout => "TIMEOUT",
            NetworkErrorKind::TooManyRedirects => "TOO_MANY_REDIRECTS",
            NetworkErrorKind::InvalidResponse => "INVALID_RESPONSE",
            NetworkErrorKind::RateLimit => "RATE_LIMIT",
            NetworkErrorKind::Other => "NETWORK_ERROR",
        }
    }
}
