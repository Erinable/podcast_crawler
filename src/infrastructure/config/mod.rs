//! Configuration management for the podcast crawler.
//!
//! This module provides a type-safe configuration system that supports:
//! - Environment variable overrides
//! - Default values
//! - Validation
//! - Test configuration

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

    pub fn validate(&self) -> AppResult<()> {
        self.database.validate()?;
        self.server.validate()?;
        self.crawler.validate()?;
        self.logging.validate()?;
        Ok(())
    }

    pub fn database_url(&self) -> &str {
        &self.database.url
    }

    pub fn database_max_connections(&self) -> u32 {
        self.database.max_connections
    }

    pub fn server_address(&self) -> String {
        format!("{}:{}", self.server.host, self.server.port)
    }

    pub fn log_level(&self) -> &str {
        &self.logging.level
    }
}

#[cfg(test)]
mod tests {}
