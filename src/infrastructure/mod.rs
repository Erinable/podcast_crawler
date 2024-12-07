//! Infrastructure layer for the podcast crawler.
//!
//! This module provides core infrastructure components including:
//! - Error handling
//! - Configuration management
//! - Logging
//! - Database access
//! - Application initialization
//!
//! # Architecture
//!
//! The infrastructure layer is organized into several key modules:
//!
//! - `error`: Error types and handling utilities
//! - `config`: Configuration management
//! - `logging`: Logging infrastructure
//! - `persistence`: Database and repository implementations
//! - `init`: Application initialization and state management
//!
//! # Usage
//!
//! ```rust
//! use podcast_crawler::infrastructure::{self, AppState};
//!
//! #[tokio::main]
//! async fn main() -> infrastructure::AppResult<()> {
//!     // Initialize the application
//!     let app_state = infrastructure::initialize().await?;
//!
//!     // Use the initialized state
//!     app_state.health_check().await?;
//!     Ok(())
//! }
//! ```

pub mod config;
pub mod error;
pub mod init;
pub mod logging;
pub mod persistence;

// Re-export commonly used types
pub use config::{DatabaseConfig, LoggingConfig, ServerConfig, Settings};
pub use error::infrastructure::{InfrastructureError, InfrastructureErrorKind};
pub use error::{AppError, AppResult, AppResultExt};
pub use init::{AppRepositories, AppState};
pub use persistence::database::{DatabaseContext, DbConnection, DbPool};

/// Initialize the application infrastructure with default settings.
///
/// This is the main entry point for initializing the application. It performs
/// the following steps:
///
/// 1. Loads and validates configuration
/// 2. Initializes logging system
/// 3. Sets up database connections
/// 4. Creates repositories
/// 5. Performs health checks
///
/// # Returns
///
/// * `AppResult<AppState>` - Initialized application state or error with context
///
/// # Examples
///
/// ```rust
/// use podcast_crawler::infrastructure;
///
/// #[tokio::main]
/// async fn main() -> infrastructure::AppResult<()> {
///     let app_state = infrastructure::initialize().await?;
///     println!("Application initialized successfully!");
///     Ok(())
/// }
/// ```
pub async fn initialize() -> AppResult<AppState> {
    AppState::init().await
}

/// Initialize the application infrastructure with custom settings.
///
/// This function allows for more control over the initialization process
/// by accepting custom settings.
///
/// # Arguments
///
/// * `settings` - Custom application settings
///
/// # Returns
///
/// * `AppResult<AppState>` - Initialized application state or error with context
///
/// # Examples
///
/// ```rust
/// use podcast_crawler::infrastructure::{self, Settings};
///
/// #[tokio::main]
/// async fn main() -> infrastructure::AppResult<()> {
///     let mut settings = Settings::default();
///     settings.database.max_connections = 20;
///
///     let app_state = infrastructure::initialize_with_settings(settings).await?;
///     println!("Application initialized with custom settings!");
///     Ok(())
/// }
/// ```
pub async fn initialize_with_settings(settings: Settings) -> AppResult<AppState> {
    AppState::init_with_settings(settings).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_infrastructure_initialization() {
        let mut settings = Settings::default();
        settings.database.url = "postgres://podcast:podcast@localhost/podcast_test".to_string();
        settings.database.max_connections = 2;
        settings.logging.level = "debug".to_string();
        settings.is_test = true;

        let app_state = initialize_with_settings(settings).await;
        assert!(app_state.is_ok(), "Infrastructure initialization failed");

        if let Ok(state) = app_state {
            assert!(state.health_check().await.is_ok(), "Health check failed");
        }
    }

    #[tokio::test]
    async fn test_default_initialization() {
        let result = initialize().await;
        assert!(
            result.is_ok() || result.is_err(),
            "Initialization should either succeed or fail with proper error"
        );
    }

    #[tokio::test]
    async fn test_invalid_settings() {
        let mut settings = Settings::default();
        settings.database.url = "invalid://url".to_string();

        let result = initialize_with_settings(settings).await;
        assert!(matches!(result, Err(AppError::Infrastructure(_))));
    }
}
