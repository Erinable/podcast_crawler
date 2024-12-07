//! Server configuration settings.
//!
//! This module provides configuration for the HTTP server including:
//! - Host address
//! - Port number
//! - Worker thread count
//!
//! # Environment Variables
//!
//! The following environment variables can be used to configure the server:
//! - `SERVER_HOST`: Server host address
//! - `SERVER_PORT`: Server port number
//! - `SERVER_WORKERS`: Number of worker threads
//!
//! # Example
//!
//! ```rust
//! use podcast_crawler::infrastructure::config::ServerConfig;
//!
//! let config = ServerConfig {
//!     host: "127.0.0.1".to_string(),
//!     port: 8080,
//!     workers: 4,
//! };
//!
//! assert!(config.validate().is_ok());
//! ```

use crate::infrastructure::AppResult;
use crate::{config_set_env, config_set_string, config_validate};
use serde::{Deserialize, Serialize};

/// Server configuration
///
/// This struct contains all the configuration settings for the HTTP server.
///
/// # Fields
///
/// * `host` - Server host address
/// * `port` - Server port number
/// * `workers` - Number of worker threads
///
/// # Default Values
///
/// - Host: "127.0.0.1"
/// - Port: 8080
/// - Workers: 4
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub workers: usize,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 8080,
            workers: 4,
        }
    }
}

impl ServerConfig {
    /// Sets configuration values from environment variables
    ///
    /// # Environment Variables
    ///
    /// - `SERVER_HOST`
    /// - `SERVER_PORT`
    /// - `SERVER_WORKERS`
    ///
    /// # Returns
    ///
    /// Returns Ok(()) if successful, or an error if environment variable parsing fails.
    pub fn set_from_env(&mut self) -> AppResult<()> {
        config_set_string!(self, "SERVER_HOST", self.host);
        config_set_env!(self, "SERVER_PORT", self.port);
        config_set_env!(self, "SERVER_WORKERS", self.workers);
        Ok(())
    }

    /// Validates the server configuration
    ///
    /// Checks that:
    /// - Host is not empty
    /// - Port is greater than 0
    /// - Worker count is greater than 0
    ///
    /// # Returns
    ///
    /// Returns Ok(()) if validation succeeds, or an error if validation fails.
    pub fn validate(&self) -> AppResult<()> {
        config_validate!(!self.host.is_empty(), "Server host cannot be empty");
        config_validate!(self.port > 0, "Server port cannot be 0");
        config_validate!(self.workers > 0, "Server workers cannot be 0");
        Ok(())
    }
}
