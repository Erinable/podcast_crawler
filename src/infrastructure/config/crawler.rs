//! Crawler configuration settings.
//!
//! This module provides configuration for the podcast crawler including:
//! - Concurrency control
//! - Rate limiting
//! - HTTP client settings
//!
//! # Environment Variables
//!
//! The following environment variables can be used to configure the crawler:
//! - `CRAWLER_USER_AGENT`: User agent string for HTTP requests
//! - `CRAWLER_MAX_TASKS`: Maximum number of concurrent crawling tasks
//! - `CRAWLER_FETCH_INTERVAL`: Interval between fetches in seconds
//!
//! # Example
//!
//! ```rust
//! use podcast_crawler::infrastructure::config::CrawlerConfig;
//!
//! let config = CrawlerConfig {
//!     max_concurrent_tasks: 5,
//!     fetch_interval_seconds: 3600,
//!     user_agent: "PodcastCrawler/1.0".to_string(),
//! };
//!
//! assert!(config.validate().is_ok());
//! ```

use crate::infrastructure::config::AppResult;
use crate::{config_set_env, config_set_string, config_validate};
use serde::{Deserialize, Serialize};

/// Crawler configuration
///
/// This struct contains all the configuration settings for the podcast crawler.
///
/// # Fields
///
/// * `max_concurrent_tasks` - Maximum number of concurrent crawling tasks
/// * `fetch_interval_seconds` - Interval between fetches in seconds
/// * `user_agent` - User agent string for HTTP requests
///
/// # Default Values
///
/// - Max Concurrent Tasks: 5
/// - Fetch Interval: 3600 seconds (1 hour)
/// - User Agent: "PodcastCrawler/1.0"
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CrawlerConfig {
    pub max_concurrent_tasks: usize,
    pub fetch_interval_seconds: u64,
    pub user_agent: String,
}

impl Default for CrawlerConfig {
    fn default() -> Self {
        Self {
            max_concurrent_tasks: 5,
            fetch_interval_seconds: 3600,
            user_agent: "PodcastCrawler/1.0".to_string(),
        }
    }
}

impl CrawlerConfig {
    /// Sets configuration values from environment variables
    ///
    /// # Environment Variables
    ///
    /// - `CRAWLER_USER_AGENT`: User agent string
    /// - `CRAWLER_MAX_TASKS`: Maximum concurrent tasks
    /// - `CRAWLER_FETCH_INTERVAL`: Fetch interval in seconds
    ///
    /// # Returns
    ///
    /// Returns Ok(()) if successful, or an error if environment variable parsing fails.
    pub fn set_from_env(&mut self) -> AppResult<()> {
        config_set_string!(self, "CRAWLER_USER_AGENT", self.user_agent);
        config_set_env!(self, "CRAWLER_MAX_TASKS", self.max_concurrent_tasks);
        config_set_env!(self, "CRAWLER_FETCH_INTERVAL", self.fetch_interval_seconds);
        Ok(())
    }

    /// Validates the crawler configuration
    ///
    /// Checks that:
    /// - Maximum concurrent tasks is greater than 0
    /// - Fetch interval is greater than 0
    /// - User agent is not empty
    ///
    /// # Returns
    ///
    /// Returns Ok(()) if validation succeeds, or an error if validation fails.
    pub fn validate(&self) -> AppResult<()> {
        config_validate!(
            self.max_concurrent_tasks > 0,
            "Max concurrent tasks must be > 0"
        );
        config_validate!(
            self.fetch_interval_seconds > 0,
            "Fetch interval must be > 0"
        );
        config_validate!(!self.user_agent.is_empty(), "User agent cannot be empty");
        Ok(())
    }
}
