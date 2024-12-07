use std::fmt;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq)]
pub enum ParseErrorKind {
    InvalidXml,
    InvalidRss,
    InvalidAtom,
    MissingField,
    InvalidFormat,
    Other,
}

impl fmt::Display for ParseErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidXml => write!(f, "Invalid XML"),
            Self::InvalidRss => write!(f, "Invalid RSS"),
            Self::InvalidAtom => write!(f, "Invalid Atom"),
            Self::MissingField => write!(f, "Missing field"),
            Self::InvalidFormat => write!(f, "Invalid format"),
            Self::Other => write!(f, "Other parse error"),
        }
    }
}

#[derive(Error, Debug)]
#[error("Parse error ({kind}): {message} for URL: {url}")]
pub struct ParseError {
    pub kind: ParseErrorKind,
    pub message: String,
    pub url: String,
    #[source]
    pub source: Option<Box<dyn std::error::Error + Send + Sync>>,
}

impl ParseError {
    pub fn new(
        kind: ParseErrorKind,
        message: impl Into<String>,
        url: impl Into<String>,
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    ) -> Self {
        Self {
            kind,
            message: message.into(),
            url: url.into(),
            source,
        }
    }

    pub fn error_code(&self) -> &'static str {
        match self.kind {
            ParseErrorKind::InvalidXml => "INVALID_XML",
            ParseErrorKind::InvalidRss => "INVALID_RSS",
            ParseErrorKind::InvalidAtom => "INVALID_ATOM",
            ParseErrorKind::MissingField => "MISSING_FIELD",
            ParseErrorKind::InvalidFormat => "INVALID_FORMAT",
            ParseErrorKind::Other => "PARSE_ERROR",
        }
    }

    pub fn is_retryable(&self) -> bool {
        matches!(self.kind, ParseErrorKind::InvalidXml | ParseErrorKind::InvalidFormat)
    }
}
