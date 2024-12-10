//! Parsing-related error types and handling.
//!
//! This module provides error types for parsing operations including:
//! - XML document parsing
//! - RSS feed parsing
//! - Atom feed parsing
//! - Field validation and extraction
//!
//! # Error Structure
//!
//! ```text
//! ┌─────────────────────────┐
//! │     ParseError          │
//! ├─────────────────────────┤
//! │ - kind: ErrorKind       │
//! │ - message: String       │
//! │ - url: String          │
//! │ - source: Option<Error> │
//! └─────────────────────────┘
//! ```
//!
//! # Example
//!
//! ```rust
//! use podcast_crawler::infrastructure::error::parse::{ParseError, ParseErrorKind};
//!
//! fn parse_feed(content: &str, url: &str) -> Result<(), ParseError> {
//!     if content.is_empty() {
//!         return Err(ParseError::new(
//!             ParseErrorKind::InvalidFormat,
//!             "Empty feed content",
//!             url,
//!             None,
//!         ));
//!     }
//!     Ok(())
//! }
//! ```

use std::fmt;
use thiserror::Error;

/// Types of parsing errors
///
/// This enum represents different categories of parsing errors
/// that can occur when processing XML, RSS, or Atom feeds.
#[derive(Debug, Clone, PartialEq)]
pub enum ParseErrorKind {
    /// Invalid XML document structure
    InvalidXml,
    /// Invalid RSS feed format
    InvalidRss,
    /// Invalid Atom feed format
    InvalidAtom,
    /// Required field is missing
    MissingField,
    /// Invalid data format
    InvalidFormat,
    /// Other parsing-related errors
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

/// Parse error type
///
/// Represents errors that occur during parsing operations,
/// such as XML parsing, RSS feed parsing, or field extraction.
///
/// # Fields
///
/// * `kind` - The type of parse error
/// * `message` - A human-readable error message
/// * `url` - The URL being parsed when the error occurred
/// * `source` - The underlying error that caused this error, if any
///
/// # Example
///
/// ```rust
/// use podcast_crawler::infrastructure::error::parse::{ParseError, ParseErrorKind};
///
/// let error = ParseError::new(
///     ParseErrorKind::InvalidRss,
///     "Missing required channel element",
///     "https://example.com/feed.xml",
///     None,
/// );
/// assert_eq!(error.error_code(), "INVALID_RSS_ERROR");
/// assert!(!error.is_retryable());
/// ```
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
    /// Creates a new parse error
    ///
    /// # Arguments
    ///
    /// * `kind` - The type of parse error
    /// * `message` - A human-readable error message
    /// * `url` - The URL being parsed when the error occurred
    /// * `source` - The underlying error that caused this error (optional)
    ///
    /// # Returns
    ///
    /// Returns a new `ParseError` instance
    ///
    /// # Example
    ///
    /// ```rust
    /// use podcast_crawler::infrastructure::error::parse::{ParseError, ParseErrorKind};
    ///
    /// let error = ParseError::new(
    ///     ParseErrorKind::MissingField,
    ///     "Required title field is missing",
    ///     "https://example.com/feed.xml",
    ///     None,
    /// );
    /// ```
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

    /// Gets the error code for this error
    ///
    /// Returns a unique code that identifies this type of error.
    /// These codes can be used in API responses or logging.
    ///
    /// # Returns
    ///
    /// Returns a static string containing the error code:
    /// - `INVALID_XML_ERROR` for XML parsing errors
    /// - `INVALID_RSS_ERROR` for RSS format errors
    /// - `INVALID_ATOM_ERROR` for Atom format errors
    /// - `MISSING_FIELD_ERROR` for missing field errors
    /// - `INVALID_FORMAT_ERROR` for format errors
    /// - `PARSE_ERROR` for other parsing errors
    pub fn error_code(&self) -> &'static str {
        match self.kind {
            ParseErrorKind::InvalidXml => "INVALID_XML_ERROR",
            ParseErrorKind::InvalidRss => "INVALID_RSS_ERROR",
            ParseErrorKind::InvalidAtom => "INVALID_ATOM_ERROR",
            ParseErrorKind::MissingField => "MISSING_FIELD_ERROR",
            ParseErrorKind::InvalidFormat => "INVALID_FORMAT_ERROR",
            ParseErrorKind::Other => "PARSE_ERROR",
        }
    }

    /// Checks if this error is retryable
    ///
    /// Determines whether the parsing operation that caused this error
    /// can be retried.
    ///
    /// # Returns
    ///
    /// Returns `true` if the operation can be retried, `false` otherwise.
    /// Currently, no parsing errors are considered retryable since they
    /// are typically due to invalid data format rather than transient issues.
    pub fn is_retryable(&self) -> bool {
        matches!(
            self.kind,
            ParseErrorKind::InvalidXml | ParseErrorKind::InvalidFormat
        )
    }
}
