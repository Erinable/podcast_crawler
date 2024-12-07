//! Configuration utility functions.
//!
//! This module provides utility functions for configuration management including:
//! - Environment variable parsing
//! - String value handling
//! - Test environment support
//!
//! # Example
//!
//! ```rust
//! use podcast_crawler::infrastructure::config::utils::{parse_env, get_env_string};
//!
//! fn load_config() -> Result<(), Box<dyn std::error::Error>> {
//!     // Parse numeric value
//!     let port: u16 = parse_env("PORT")?;
//!
//!     // Get string value
//!     let host = get_env_string("HOST")?;
//!
//!     println!("Server will listen on {}:{}", host, port);
//!     Ok(())
//! }
//! ```

use crate::infrastructure::{AppError, AppResult, InfrastructureError, InfrastructureErrorKind};

/// Parses an environment variable into a specified type
///
/// This function attempts to parse an environment variable into any type that
/// implements the `FromStr` trait.
///
/// # Type Parameters
///
/// * `T` - The target type to parse into, must implement `FromStr`
///
/// # Arguments
///
/// * `env_var` - The name of the environment variable to parse
///
/// # Returns
///
/// Returns a Result containing the parsed value if successful,
/// or an error if the environment variable is not found or parsing fails.
///
/// # Example
///
/// ```rust
/// use podcast_crawler::infrastructure::config::utils::parse_env;
///
/// let port: u16 = parse_env("PORT")?;
/// assert!(port > 0);
/// ```
pub fn parse_env<T: std::str::FromStr>(env_var: &str) -> AppResult<T> {
    let value = std::env::var(env_var).map_err(|e| {
        let infra_err: InfrastructureError = InfrastructureError::new(
            InfrastructureErrorKind::Config,
            format!("Failed to read environment variable: {}", env_var),
            Some(Box::new(e)),
        );
        AppError::from(infra_err)
    })?;

    value.parse().map_err(|_| {
        let infra_err: InfrastructureError = InfrastructureError::new(
            InfrastructureErrorKind::Config,
            format!(
                "Failed to parse environment variable: {} with value: {}",
                env_var, value
            ),
            None,
        );
        AppError::from(infra_err)
    })
}

/// Gets a string value from an environment variable
///
/// This function retrieves a string value from the specified environment variable.
///
/// # Arguments
///
/// * `env_var` - The name of the environment variable to read
///
/// # Returns
///
/// Returns a Result containing the string value if successful,
/// or an error if the environment variable is not found.
///
/// # Example
///
/// ```rust
/// use podcast_crawler::infrastructure::config::utils::get_env_string;
///
/// let host = get_env_string("HOST")?;
/// assert!(!host.is_empty());
/// ```
pub fn get_env_string(env_var: &str) -> AppResult<String> {
    std::env::var(env_var).map_err(|e| {
        let infra_err: InfrastructureError = InfrastructureError::new(
            InfrastructureErrorKind::Config,
            format!("Failed to read environment variable: {}", env_var),
            Some(Box::new(e)),
        );
        AppError::from(infra_err)
    })
}

/// Gets a string value from an environment variable with test support
///
/// This function retrieves a string value from either a regular environment
/// variable or a test-specific environment variable based on the test flag.
///
/// # Arguments
///
/// * `env_var` - The name of the regular environment variable
/// * `test_env_var` - The name of the test environment variable
/// * `is_test` - Flag indicating whether to use the test variable
///
/// # Returns
///
/// Returns a Result containing the string value if successful,
/// or an error if the environment variable is not found.
///
/// # Example
///
/// ```rust
/// use podcast_crawler::infrastructure::config::utils::get_env_string_with_test;
///
/// let db_url = get_env_string_with_test("DATABASE_URL", "TEST_DATABASE_URL", true)?;
/// assert!(!db_url.is_empty());
/// ```
pub fn get_env_string_with_test(
    env_var: &str,
    test_env_var: &str,
    is_test: bool,
) -> AppResult<String> {
    let target_var = if is_test { test_env_var } else { env_var };
    std::env::var(target_var).map_err(|e| {
        let infra_err: InfrastructureError = InfrastructureError::new(
            InfrastructureErrorKind::Config,
            format!("Failed to read environment variable: {}", target_var),
            Some(Box::new(e)),
        );
        AppError::from(infra_err)
    })
}
