//! Logging infrastructure for the podcast crawler application.
//!
//! This module provides a flexible logging system built on top of `tracing`,
//! supporting multiple output formats and destinations.
//!
//! # Features
//!
//! - Multiple log formats (JSON, Text)
//! - File and console output
//! - Log rotation
//! - Log levels
//! - Structured logging
//! - Request tracing
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────┐
//! │  LoggingConfig  │
//! ├─────────────────┤
//! │ Format          │
//! │ Level           │
//! │ Path            │
//! │ Rotation        │
//! └─────────────────┘
//! ```
//!
//! # Example
//!
//! ```rust
//! use podcast_crawler::infrastructure::{
//!     config::LoggingConfig,
//!     logging::init_logger,
//! };
//!
//! fn setup_logging() -> Result<(), Box<dyn std::error::Error>> {
//!     let config = LoggingConfig {
//!         format: "json".to_string(),
//!         level: "info".to_string(),
//!         path: Some("logs".to_string()),
//!         ..Default::default()
//!     };
//!     init_logger(&config)?;
//!     Ok(())
//! }
//! ```

use std::{path::Path, sync::Once};
use time::macros::format_description;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{
    fmt::{
        self,
        format::{FmtSpan, Format},
        time::LocalTime,
    },
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter,
};

use crate::infrastructure::{config::LoggingConfig, error::AppResult};

static LOGGER_INIT: Once = Once::new();

/// Initialize the logging system with the provided configuration
///
/// This function sets up the logging system according to the provided configuration.
/// It ensures that the logger is only initialized once using a static `Once` guard.
///
/// # Features
///
/// - Multiple output formats (JSON, Text)
/// - Console and file output
/// - Log rotation (Daily, Hourly)
/// - Configurable log levels
/// - Request span tracking
///
/// # Arguments
///
/// * `config` - Logging configuration containing format, level, and path settings
///
/// # Returns
///
/// Returns `AppResult<()>` which is:
/// - `Ok(())` if logger initialization succeeds
/// - `Err(AppError)` if initialization fails
///
/// # Example
///
/// ```rust
/// use podcast_crawler::infrastructure::{
///     config::LoggingConfig,
///     logging::init_logger,
/// };
///
/// let config = LoggingConfig {
///     format: "json".to_string(),
///     level: "info".to_string(),
///     path: Some("logs".to_string()),
///     ..Default::default()
/// };
/// init_logger(&config)?;
/// ```
pub fn init_logger(config: &LoggingConfig) -> AppResult<()> {
    // Skip if logger is already initialized
    if LOGGER_INIT.is_completed() {
        return Ok(());
    }

    // Initialize with both layers
    LOGGER_INIT.call_once(|| {
        // Create file appender
        let file_appender = RollingFileAppender::new(
            Rotation::DAILY,
            Path::new(&config.file_path)
                .parent()
                .unwrap_or(Path::new(".")),
            Path::new(&config.file_path)
                .file_name()
                .unwrap_or_default()
                .to_str()
                .unwrap_or("podcast_crawler.log"),
        );

        // Create time format
        let time_format = format_description!("[year]-[month]-[day] [hour]:[minute]:[second]");
        let timer = LocalTime::new(time_format);

        // Create file layer based on format
        let file_layer: Box<dyn tracing_subscriber::Layer<_> + Send + Sync> = if config.json_format
        {
            Box::new(
                fmt::layer()
                    .json()
                    .with_writer(file_appender)
                    .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
                    .with_target(true)
                    .with_thread_ids(true)
                    .with_thread_names(true)
                    .with_file(true)
                    .with_line_number(true),
            )
        } else {
            Box::new(
                fmt::layer()
                    .with_writer(file_appender)
                    .event_format(
                        Format::default()
                            .with_level(true)
                            .with_target(true)
                            .with_thread_ids(false)
                            .with_thread_names(false)
                            .with_file(false)
                            .with_line_number(true)
                            .with_ansi(true)
                            .with_source_location(true)
                            .with_timer(timer.clone())
                            .compact(),
                    )
                    .with_ansi(true),
            )
        };

        // Create env filter with different log levels for different modules
        let env_filter = EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new(&format!("{}", config.level)))
            // Set specific log levels for external crates
            .add_directive("tokio_postgres=warn".parse().unwrap())
            .add_directive("tokio=warn".parse().unwrap())
            .add_directive("runtime=warn".parse().unwrap())
            .add_directive("hyper=warn".parse().unwrap())
            .add_directive("sqlx=warn".parse().unwrap())
            .add_directive("reqwest=warn".parse().unwrap())
            .add_directive("html5ever=warn".parse().unwrap());

        // Create console layer with colored output
        let stdout_layer = fmt::layer()
            .with_writer(std::io::stdout)
            .event_format(
                Format::default()
                    .with_level(true)
                    .with_target(true)
                    .with_thread_ids(false)
                    .with_thread_names(false)
                    .with_file(false)
                    .with_line_number(true)
                    .with_ansi(true)
                    .with_source_location(true)
                    .with_timer(timer)
                    .compact(),
            )
            .with_ansi(true);

        // Register subscriber with both layers and filter
        tracing_subscriber::registry()
            .with(env_filter)
            .with(file_layer)
            .with(stdout_layer)
            .init();
    });

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    /// Test JSON format logging initialization
    #[test]
    fn test_init_logger_with_json() {
        let config = LoggingConfig {
            level: "debug".to_string(),
            file_path: "target/test-logs-json".to_string(),
            json_format: true,
        };

        assert!(init_logger(&config).is_ok());
        assert!(Path::new("target/test-logs-json").exists());
        fs::remove_dir_all("target/test-logs-json").unwrap();
    }

    /// Test text format logging initialization
    #[test]
    fn test_init_logger_with_text() {
        let config = LoggingConfig {
            level: "debug".to_string(),
            file_path: "target/test-logs-text".to_string(),
            json_format: false,
        };

        assert!(init_logger(&config).is_ok());
        assert!(Path::new("target/test-logs-text").exists());
        fs::remove_dir_all("target/test-logs-text").unwrap();
    }

    /// Test logging initialization with invalid path
    #[test]
    fn test_init_logger_invalid_path() {
        let config = LoggingConfig {
            level: "debug".to_string(),
            file_path: "/invalid/path/that/should/not/exist".to_string(),
            json_format: false,
        };

        assert!(init_logger(&config).is_err());
    }
}
