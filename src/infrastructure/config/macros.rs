//! Configuration macros.
//!
//! This module provides macros for common configuration operations including:
//! - Environment variable parsing
//! - Configuration validation
//! - String value handling
//!
//! # Example
//!
//! ```rust
//! use podcast_crawler::{config_set_env, config_validate};
//!
//! struct Config {
//!     port: u16,
//! }
//!
//! impl Config {
//!     fn set_from_env(&mut self) -> Result<(), Box<dyn std::error::Error>> {
//!         config_set_env!(self, "PORT", self.port);
//!         Ok(())
//!     }
//!
//!     fn validate(&self) -> Result<(), Box<dyn std::error::Error>> {
//!         config_validate!(self.port > 0, "Port must be greater than 0");
//!         Ok(())
//!     }
//! }
//! ```

/// Sets a configuration value from an environment variable
///
/// This macro attempts to parse an environment variable into the target type.
/// If parsing fails, returns an error.
///
/// # Arguments
///
/// * `$settings` - The configuration struct being modified
/// * `$env` - The environment variable name
/// * `$target` - The target field to set
///
/// # Example
///
/// ```rust
/// config_set_env!(self, "PORT", self.port);
/// ```
#[macro_export]
macro_rules! config_set_env {
    ($settings:expr, $env:literal, $target:expr) => {
        match $crate::infrastructure::config::utils::parse_env($env) {
            Ok(value) => $target = value,
            Err(e) => return Err(e),
        }
    };
}

/// Sets a string configuration value from an environment variable
///
/// This macro attempts to get a string value from an environment variable.
/// If the environment variable is not found, returns an error.
///
/// # Arguments
///
/// * `$settings` - The configuration struct being modified
/// * `$env` - The environment variable name
/// * `$target` - The target field to set
///
/// # Example
///
/// ```rust
/// config_set_string!(self, "HOST", self.host);
/// ```
#[macro_export]
macro_rules! config_set_string {
    ($settings:expr, $env:literal, $target:expr) => {
        match $crate::infrastructure::config::utils::get_env_string($env) {
            Ok(value) => $target = value,
            Err(e) => return Err(e),
        }
    };
}

/// Validates a configuration condition
///
/// This macro checks a condition and returns an error with the specified message
/// if the condition is false.
///
/// # Arguments
///
/// * `$cond` - The condition to check
/// * `$msg` - The error message if condition is false
///
/// # Example
///
/// ```rust
/// config_validate!(port > 0, "Port must be greater than 0");
/// ```
#[macro_export]
macro_rules! config_validate {
    ($cond:expr, $msg:expr) => {
        if !$cond {
            return Err($crate::infrastructure::AppError::Infrastructure(
                $crate::infrastructure::InfrastructureError::new(
                    $crate::infrastructure::InfrastructureErrorKind::Config,
                    $msg,
                    None,
                ),
            ));
        }
    };
}
