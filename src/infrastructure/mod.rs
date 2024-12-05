pub mod config;
pub mod error;
pub mod init;
pub mod logging;
pub mod persistence;

// Re-export commonly used types
pub use config::{LoggingConfig, Settings};
pub use error::{AppError, AppResult, AppResultExt};
pub use init::AppState;

// Initialize function for the entire infrastructure
pub async fn initialize() -> AppResult<AppState> {
    AppState::init().await
}
