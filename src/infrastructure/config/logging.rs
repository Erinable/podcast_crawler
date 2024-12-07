//! Logging configuration settings.
//!
//! This module provides configuration for the application logging including:
//! - Log level
//! - Log file path
//! - Log format (plain text or JSON)
//!
//! # Environment Variables
//!
//! The following environment variables can be used to configure logging:
//! - `LOG_LEVEL`: Log level (error, warn, info, debug, trace)
//! - `LOG_FILE`: Path to log file
//! - `LOG_JSON`: Whether to use JSON format (true/false)
//!
//! # Example
//!
//! ```rust
//! use podcast_crawler::infrastructure::config::LoggingConfig;
//!
//! let config = LoggingConfig {
//!     level: "info".to_string(),
//!     file_path: "logs".to_string(),
//!     json_format: false,
//! };
//!
//! assert!(config.validate().is_ok());
//! ```

use crate::infrastructure::config::AppResult;
use crate::{config_set_env, config_set_string, config_validate};
use serde::{Deserialize, Serialize};

/// Logging configuration
///
/// This struct contains all the configuration settings for application logging.
///
/// # Fields
///
/// * `level` - Log level (error, warn, info, debug, trace)
/// * `file_path` - Path to log file
/// * `json_format` - Whether to use JSON format for logs
///
/// # Default Values
///
/// - Level: "info"
/// - File Path: "logs"
/// - JSON Format: false
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LoggingConfig {
    pub level: String,
    pub file_path: String,
    pub json_format: bool,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            file_path: "logs".to_string(),
            json_format: false,
        }
    }
}

impl LoggingConfig {
    /// Sets configuration values from environment variables
    ///
    /// # Environment Variables
    ///
    /// - `LOG_LEVEL`: Log level
    /// - `LOG_FILE`: Log file path
    /// - `LOG_JSON`: JSON format flag
    ///
    /// # Returns
    ///
    /// Returns Ok(()) if successful, or an error if environment variable parsing fails.
    /// Automatically validates the configuration after setting values.
    pub fn set_from_env(&mut self) -> AppResult<()> {
        config_set_string!(self, "LOG_LEVEL", self.level);
        config_set_string!(self, "LOG_FILE", self.file_path);
        config_set_env!(self, "LOG_JSON", self.json_format);
        self.validate()?;
        Ok(())
    }

    /// Validates the logging configuration
    ///
    /// Checks that:
    /// - Log level is not empty
    /// - Log level is one of: error, warn, info, debug, trace
    /// - Log file path is not empty
    ///
    /// # Returns
    ///
    /// Returns Ok(()) if validation succeeds, or an error if validation fails.
    pub fn validate(&self) -> AppResult<()> {
        config_validate!(!self.level.is_empty(), "Log level cannot be empty");
        config_validate!(
            matches!(
                self.level.as_str(),
                "error" | "warn" | "info" | "debug" | "trace"
            ),
            "Invalid log level. Must be one of: error, warn, info, debug, trace"
        );
        config_validate!(!self.file_path.is_empty(), "Log file path cannot be empty");
        Ok(())
    }
}
