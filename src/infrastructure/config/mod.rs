//! Configuration management for the podcast crawler.
//!
//! This module provides a type-safe configuration system that supports:
//! - Environment variable overrides
//! - Default values
//! - Validation
//! - Test configuration
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────┐
//! │      Settings       │
//! ├─────────────────────┤
//! │ - database         │
//! │ - server          │
//! │ - crawler         │
//! │ - logging         │
//! │ - is_test         │
//! └─────────────────────┘
//!         ▲
//!         │
//!    ┌────┴────┐
//! ┌──┴──┐ ┌──┴──┐
//! │ Env │ │File │
//! └─────┘ └─────┘
//! ```
//!
//! # Example
//!
//! ```rust
//! use podcast_crawler::infrastructure::config::Settings;
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Load configuration from environment and defaults
//!     let settings = Settings::new()?;
//!
//!     // Validate configuration
//!     settings.validate()?;
//!
//!     // Use configuration values
//!     println!("Database URL: {}", settings.database_url());
//!     println!("Server Address: {}", settings.server_address());
//!
//!     Ok(())
//! }
//! ```

use crate::config_set_env;
use crate::infrastructure::AppResult;
use serde::{Deserialize, Serialize};

pub mod crawler;
pub mod database;
pub mod logging;
pub mod macros;
pub mod server;
pub mod utils;

pub use crawler::CrawlerConfig;
pub use database::DatabaseConfig;
pub use logging::LoggingConfig;
pub use server::ServerConfig;

/// Application configuration settings
///
/// This struct represents the complete configuration for the podcast crawler,
/// including database, server, crawler, and logging settings.
///
/// # Fields
///
/// * `database` - Database connection and pool settings
/// * `server` - HTTP server configuration
/// * `crawler` - Podcast crawler settings
/// * `logging` - Logging configuration
/// * `is_test` - Flag indicating if running in test mode
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Settings {
    pub database: DatabaseConfig,
    pub server: ServerConfig,
    pub crawler: CrawlerConfig,
    pub logging: LoggingConfig,
    pub is_test: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            database: DatabaseConfig::default(),
            server: ServerConfig::default(),
            crawler: CrawlerConfig::default(),
            logging: LoggingConfig::default(),
            is_test: false,
        }
    }
}

impl Settings {
    /// Creates a new Settings instance
    ///
    /// Loads configuration from environment variables and default values.
    /// Uses the following precedence (highest to lowest):
    /// 1. Environment variables
    /// 2. Default values
    ///
    /// # Returns
    ///
    /// Returns a Result containing the Settings instance if successful,
    /// or an error if configuration loading fails.
    ///
    /// # Example
    ///
    /// ```rust
    /// use podcast_crawler::infrastructure::config::Settings;
    ///
    /// let settings = Settings::new()?;
    /// assert!(!settings.is_test);
    /// ```
    pub fn new() -> AppResult<Self> {
        dotenv::dotenv().ok();
        let mut settings = Self::default();

        // Test flag
        config_set_env!(settings, "TEST_FLAG", settings.is_test);

        // Set configurations from environment for each component
        settings.database.set_from_env(settings.is_test)?;
        settings.server.set_from_env()?;
        settings.crawler.set_from_env()?;
        settings.logging.set_from_env()?;

        settings.validate()?;
        Ok(settings)
    }

    /// Validates the configuration settings
    ///
    /// Checks that all required settings are present and valid.
    /// This includes:
    /// - Database connection settings
    /// - Server configuration
    /// - Crawler settings
    /// - Logging configuration
    ///
    /// # Returns
    ///
    /// Returns Ok(()) if validation succeeds, or an error if validation fails.
    pub fn validate(&self) -> AppResult<()> {
        self.database.validate()?;
        self.server.validate()?;
        self.crawler.validate()?;
        self.logging.validate()?;
        Ok(())
    }

    /// Gets the database connection URL
    ///
    /// # Returns
    ///
    /// Returns the configured database URL as a string slice.
    pub fn database_url(&self) -> &str {
        &self.database.url
    }

    /// Gets the maximum number of database connections
    ///
    /// # Returns
    ///
    /// Returns the configured maximum number of database connections.
    pub fn database_max_connections(&self) -> u32 {
        self.database.max_connections
    }

    /// Gets the server address
    ///
    /// # Returns
    ///
    /// Returns the configured server address as a String.
    pub fn server_address(&self) -> String {
        format!("{}:{}", self.server.host, self.server.port)
    }

    /// Gets the log level
    ///
    /// # Returns
    ///
    /// Returns the configured log level as a string slice.
    pub fn log_level(&self) -> &str {
        &self.logging.level
    }
}

#[cfg(test)]
mod tests {}
