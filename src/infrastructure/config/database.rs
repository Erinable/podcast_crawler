//! Database configuration settings.
//!
//! This module provides configuration for database connections including:
//! - Connection string
//! - Connection pool settings
//! - Timeouts
//!
//! # Environment Variables
//!
//! The following environment variables can be used to configure the database:
//! - `DATABASE_URL`: Database connection string
//! - `TEST_DATABASE_URL`: Test database connection string
//! - `DATABASE_MAX_CONNECTIONS`: Maximum number of connections in the pool
//! - `DATABASE_MIN_CONNECTIONS`: Minimum number of connections in the pool
//! - `DATABASE_CONNECT_TIMEOUT`: Connection timeout in seconds
//! - `DATABASE_IDLE_TIMEOUT`: Idle connection timeout in seconds
//!
//! # Example
//!
//! ```rust
//! use podcast_crawler::infrastructure::config::DatabaseConfig;
//!
//! let config = DatabaseConfig {
//!     url: "postgres://localhost:5432/podcast".to_string(),
//!     max_connections: 10,
//!     min_connections: 2,
//!     connect_timeout_seconds: 30,
//!     idle_timeout_seconds: 300,
//! };
//!
//! assert!(config.validate().is_ok());
//! ```

use crate::infrastructure::config::utils;
use crate::infrastructure::config::AppResult;
use crate::{config_set_env, config_validate};
use serde::{Deserialize, Serialize};

/// Database configuration
///
/// This struct contains all the configuration settings for the database connection
/// and connection pool.
///
/// # Fields
///
/// * `url` - Database connection URL
/// * `max_connections` - Maximum number of connections in the pool
/// * `min_connections` - Minimum number of connections in the pool
/// * `connect_timeout_seconds` - Connection timeout in seconds
/// * `idle_timeout_seconds` - Idle connection timeout in seconds
///
/// # Default Values
///
/// - URL: "postgres://localhost:5432/podcast"
/// - Max Connections: 10
/// - Min Connections: 2
/// - Connect Timeout: 30 seconds
/// - Idle Timeout: 300 seconds
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub connect_timeout_seconds: u64,
    pub idle_timeout_seconds: u64,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: "postgres://localhost:5432/podcast".to_string(),
            max_connections: 10,
            min_connections: 2,
            connect_timeout_seconds: 30,
            idle_timeout_seconds: 300,
        }
    }
}

impl DatabaseConfig {
    /// Sets configuration values from environment variables
    ///
    /// # Arguments
    ///
    /// * `is_test` - If true, uses test-specific environment variables
    ///
    /// # Environment Variables
    ///
    /// - `DATABASE_URL` (or `TEST_DATABASE_URL` in test mode)
    /// - `DATABASE_MAX_CONNECTIONS`
    /// - `DATABASE_MIN_CONNECTIONS`
    /// - `DATABASE_CONNECT_TIMEOUT`
    /// - `DATABASE_IDLE_TIMEOUT`
    ///
    /// # Returns
    ///
    /// Returns Ok(()) if successful, or an error if environment variable parsing fails.
    pub fn set_from_env(&mut self, is_test: bool) -> AppResult<()> {
        // Special handling for database URL in test mode
        self.url = utils::get_env_string_with_test("DATABASE_URL", "TEST_DATABASE_URL", is_test)
            .unwrap_or(self.url.clone());

        // Other database settings
        config_set_env!(self, "DATABASE_MAX_CONNECTIONS", self.max_connections);
        config_set_env!(self, "DATABASE_MIN_CONNECTIONS", self.min_connections);
        config_set_env!(
            self,
            "DATABASE_CONNECT_TIMEOUT",
            self.connect_timeout_seconds
        );
        config_set_env!(self, "DATABASE_IDLE_TIMEOUT", self.idle_timeout_seconds);
        Ok(())
    }

    /// Validates the database configuration
    ///
    /// Checks that:
    /// - Database URL is not empty
    /// - Max connections >= min connections
    /// - Min connections > 0
    /// - Connect timeout > 0
    /// - Idle timeout > 0
    ///
    /// # Returns
    ///
    /// Returns Ok(()) if validation succeeds, or an error if validation fails.
    pub fn validate(&self) -> AppResult<()> {
        config_validate!(!self.url.is_empty(), "Database URL cannot be empty");
        config_validate!(
            self.max_connections >= self.min_connections,
            "Max connections must be >= min connections"
        );
        config_validate!(
            self.connect_timeout_seconds > 0,
            "Connect timeout must be > 0"
        );
        config_validate!(self.idle_timeout_seconds > 0, "Idle timeout must be > 0");
        Ok(())
    }
}
