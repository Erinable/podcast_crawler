//! Crawler module for fetching and processing podcast data.
//!
//! This module provides functionality for:
//! - HTTP crawling with rate limiting
//! - RSS feed parsing
//! - Batch processing
//! - URL handling and validation
//!
//! # Architecture
//!
//! The crawler is organized into several components:
//! - `HttpCrawler`: Main crawler implementation
//! - `RateLimiter`: Rate limiting for HTTP requests
//! - `BatchProcessor`: Batch processing of crawl tasks
//! - `RssParser`: RSS feed parsing
//!
//! # Example
//!
//! ```rust
//! use podcast_crawler::crawler::{HttpCrawler, TaskResult};
//!
//! let crawler = HttpCrawler::new(config);
//! let result = crawler.crawl("https://example.com/feed.xml").await?;
//! ```

mod batch_processor;
mod crawler_impl;
pub mod rate_limiter;
pub mod rss;
pub mod traits;
pub mod url_utils;

use std::fmt::Debug;
use std::time::Duration;

use crate::infrastructure::error::{AppError, AppResult, DomainError, DomainErrorKind};

pub use crawler_impl::HttpCrawler;
pub use traits::{Crawler, FeedParser};

/// Result of a crawling task
#[derive(Debug, Clone)]
pub struct TaskResult<T> {
    /// URL that was crawled
    pub url: String,
    /// Whether the crawl was successful
    pub success: bool,
    /// Parsed data if successful
    pub parsed_data: Option<T>,
    /// Error message if unsuccessful
    pub error_message: Option<String>,
    /// Duration of the crawl
    pub duration: Duration,
}

impl<T> TaskResult<T> {
    /// Create a new successful task result
    ///
    /// # Arguments
    /// * `url` - URL that was crawled
    /// * `parsed_data` - Successfully parsed data
    /// * `duration` - Duration of the crawl
    ///
    /// # Returns
    /// * `TaskResult<T>` - Successful task result
    pub fn success(url: String, parsed_data: T, duration: Duration) -> Self {
        Self {
            url,
            success: true,
            parsed_data: Some(parsed_data),
            error_message: None,
            duration,
        }
    }

    /// Create a new failed task result
    ///
    /// # Arguments
    /// * `url` - URL that was crawled
    /// * `error` - Error that occurred
    /// * `duration` - Duration of the crawl
    ///
    /// # Returns
    /// * `TaskResult<T>` - Failed task result
    pub fn failure(url: String, error: impl Into<String>, duration: Duration) -> Self {
        Self {
            url,
            success: false,
            parsed_data: None,
            error_message: Some(error.into()),
            duration,
        }
    }

    /// Convert the task result into a domain result
    ///
    /// # Returns
    /// * `AppResult<T>` - Domain result
    pub fn into_result(self) -> AppResult<T> {
        if self.success {
            self.parsed_data.ok_or_else(|| {
                DomainError::new(
                    DomainErrorKind::Other,
                    "Successful task with no parsed data".to_string(),
                    Some(format!("URL: {}", self.url)),
                    None,
                )
                .into()
            })
        } else {
            Err(DomainError::new(
                DomainErrorKind::Validation,
                &self
                    .error_message
                    .unwrap_or_else(|| "Unknown error".to_string()),
                Some(format!("URL: {}", self.url)),
                None,
            )
            .into())
        }
    }

    /// Check if the task was successful
    pub fn is_success(&self) -> bool {
        self.success
    }

    /// Get the duration of the task
    pub fn duration(&self) -> Duration {
        self.duration
    }

    /// Get the URL that was crawled
    pub fn url(&self) -> &str {
        &self.url
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{result, time::Duration};

    #[test]
    fn test_task_result_success() {
        let url = "https://example.com";
        let data = "test data";
        let duration = Duration::from_secs(1);

        let result = TaskResult::success(url.to_string(), data, duration);

        assert!(result.is_success());
        assert_eq!(result.url(), url);
        assert_eq!(result.duration(), duration);
        assert_eq!(result.parsed_data.unwrap(), data);
        assert!(result.error_message.is_none());

        let domain_result = result.into_result();
        assert!(domain_result.is_ok());
        assert_eq!(domain_result.unwrap(), data);
    }

    #[test]
    fn test_task_result_failure() {
        let url = "https://example.com";
        let error = "test error";
        let duration = Duration::from_secs(1);

        let result: TaskResult<()> = TaskResult::failure(url.to_string(), error, duration);

        assert!(!result.is_success());
        assert_eq!(result.url(), url);
        assert_eq!(result.duration(), duration);
        assert!(result.parsed_data.is_none());
        assert_eq!(result.error_message.unwrap(), error);

        // let domain_result = result.into_result();
        // assert!(domain_result.is_err());
        // let err = domain_result.unwrap_err();
        // assert!(err.to_string().contains(error));
        // assert!(err.to_string().contains(url));
    }

    #[test]
    fn test_task_result_empty_success() {
        let url = "https://example.com";
        let data = ();
        let duration = Duration::from_secs(1);

        let result = TaskResult::success(url.to_string(), data, duration);

        assert!(result.is_success());
        assert_eq!(result.url(), url);
        assert_eq!(result.duration(), duration);
        assert!(result.parsed_data.is_some());
        assert!(result.error_message.is_none());

        let domain_result = result.into_result();
        assert!(domain_result.is_ok());
    }
}
