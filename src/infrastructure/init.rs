//! Application initialization and state management.
//!
//! This module provides functionality for initializing and managing the application state,
//! including:
//! - Database connections
//! - Repositories
//! - Logging
//! - Configuration

use crate::infrastructure::logging::init_logger;
use crate::infrastructure::persistence::repositories::{
    EpisodeRepository, PodcastRankRepository, PodcastRepository,
};
use crate::infrastructure::Settings;
use crate::infrastructure::{
    error::infrastructure::{InfrastructureError, InfrastructureErrorKind},
    AppError, AppResult,
};

use std::sync::Arc;
use tracing::{error, info};

use super::DatabaseContext;

/// Application repositories container
#[derive(Debug)]
pub struct AppRepositories {
    pub podcast: PodcastRepository,
    pub podcast_rank: PodcastRankRepository,
    pub episode: EpisodeRepository,
}

impl AppRepositories {
    fn new(database_context: Arc<DatabaseContext>) -> Self {
        Self {
            podcast: PodcastRepository::new(database_context.clone()),
            podcast_rank: PodcastRankRepository::new(database_context.clone()),
            episode: EpisodeRepository::new(database_context),
        }
    }
}

/// Application state containing all initialized components
#[derive(Debug)]
pub struct AppState {
    pub repositories: Arc<AppRepositories>,
    pub database_context: Arc<DatabaseContext>,
    pub settings: Arc<Settings>,
}

impl AppState {
    /// Initialize the application state with provided settings
    pub async fn init_with_settings(settings: Settings) -> AppResult<Self> {
        // Validate settings first
        settings.validate().map_err(|e| {
            error!("Failed to validate settings: {}", e);
            AppError::Infrastructure(InfrastructureError::new(
                InfrastructureErrorKind::Config,
                "Failed to validate settings".to_string(),
                Some(Box::new(e)),
            ))
        })?;

        // Initialize configured logging system
        init_logger(&settings.logging).map_err(|e| {
            error!("Failed to initialize logging: {}", e);
            AppError::Infrastructure(InfrastructureError::new(
                InfrastructureErrorKind::IO,
                "Failed to initialize logging".to_string(),
                Some(Box::new(e)),
            ))
        })?;

        info!("Initializing application with custom settings...");

        // Initialize database connection pool
        info!("Initializing database connection pool...");
        let database_context = match DatabaseContext::new_with_config(&settings.database).await {
            Ok(ctx) => Arc::new(ctx),
            Err(e) => {
                error!("Failed to initialize database context: {}", e);
                return Err(AppError::Infrastructure(InfrastructureError::new(
                    InfrastructureErrorKind::Database,
                    "Failed to initialize database context".to_string(),
                    Some(Box::new(e)),
                )));
            }
        };

        // Test database connection
        info!("Testing database connection...");
        if let Err(e) = database_context.get_connection().await {
            error!("Failed to establish database connection: {}", e);
            return Err(AppError::Infrastructure(InfrastructureError::new(
                InfrastructureErrorKind::Database,
                "Failed to establish database connection".to_string(),
                Some(Box::new(e)),
            )));
        }

        // Initialize repositories
        info!("Initializing repositories...");
        let repositories = Arc::new(AppRepositories::new(database_context.clone()));

        info!("Application initialization complete!");

        Ok(Self {
            repositories,
            database_context,
            settings: Arc::new(settings),
        })
    }

    /// Initialize the application state with default settings
    pub async fn init() -> AppResult<Self> {
        let settings = Settings::new().map_err(|e| {
            AppError::Infrastructure(InfrastructureError::new(
                InfrastructureErrorKind::Config,
                "Failed to load settings".to_string(),
                Some(Box::new(e)),
            ))
        })?;
        Self::init_with_settings(settings).await
    }

    /// Checks if all components are healthy
    pub async fn health_check(&self) -> AppResult<()> {
        // Check database connection
        self.database_context.get_connection().await.map_err(|e| {
            AppError::Infrastructure(InfrastructureError::new(
                InfrastructureErrorKind::Database,
                "Database connection check failed".to_string(),
                Some(Box::new(e)),
            ))
        })?;

        // Basic repository checks
        self.repositories.podcast.get_all().await.map_err(|e| {
            AppError::Infrastructure(InfrastructureError::new(
                InfrastructureErrorKind::Database,
                "Podcast repository check failed".to_string(),
                Some(Box::new(e)),
            ))
        })?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn setup() -> Settings {
        let mut settings = Settings::default();
        settings.database.url = "postgres://podcast:podcast@localhost/podcast_test".to_string();
        settings.database.max_connections = 2;
        settings.database.min_connections = 1;
        settings.logging.level = "debug".to_string();
        settings.is_test = true;
        settings
    }

    #[tokio::test]
    async fn test_app_initialization() {
        let settings = setup().await;
        let state_result = AppState::init_with_settings(settings).await;
        assert!(state_result.is_ok(), "App initialization failed");
    }

    #[tokio::test]
    async fn test_invalid_database_url() {
        let mut settings = setup().await;
        settings.database.url = "postgres://invalid:5432/nonexistent".to_string();

        let result = AppState::init_with_settings(settings).await;
        assert!(matches!(result, Err(AppError::Infrastructure(_))));
    }

    #[tokio::test]
    async fn test_repository_operations() {
        let settings = setup().await;
        let app_state = AppState::init_with_settings(settings)
            .await
            .expect("Failed to initialize app state");

        let health_check = app_state.health_check().await;
        assert!(health_check.is_ok(), "Health check failed");
    }

    #[tokio::test]
    async fn test_concurrent_access() {
        let settings = setup().await;
        let app_state = Arc::new(
            AppState::init_with_settings(settings)
                .await
                .expect("Failed to initialize app state"),
        );

        let mut handles = vec![];
        for _ in 0..5 {
            let state = app_state.clone();
            let handle = tokio::spawn(async move {
                let health_check = state.health_check().await;
                assert!(health_check.is_ok(), "Concurrent health check failed");
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.await.unwrap();
        }
    }
}
