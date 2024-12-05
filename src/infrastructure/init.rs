use crate::infrastructure::error::AppResult;
use crate::infrastructure::logging::init_logger;
use crate::infrastructure::persistence::database::DatabaseContext;
use crate::infrastructure::persistence::repositories::{
    EpisodeRepository, PodcastRankRepository, PodcastRepository,
};

use crate::infrastructure::LoggingConfig;
use crate::infrastructure::Settings;
use std::sync::Arc;
use tracing::info;

/// Application repositories container
pub struct AppRepositories {
    pub podcast: PodcastRepository,
    pub podcast_rank: PodcastRankRepository,
    pub episode: EpisodeRepository,
}

/// Application state containing all initialized components

pub struct AppState {
    pub repositories: Arc<AppRepositories>,
    pub database_context: Arc<DatabaseContext>,
    pub settings: Arc<Settings>,
}

impl AppState {
    /// Initialize the application state with settings
    pub async fn init_with_settings(settings: Settings) -> AppResult<Self> {
        // Validate settings
        settings.validate()?;

        // Initialize logging first
        let log_config = LoggingConfig {
            level: settings.logging.level.clone(),
            file_path: settings.logging.file_path.clone(),
            json_format: settings.logging.json_format,
        };
        init_logger(&log_config)?;

        info!("Initializing application with custom settings...");

        // Initialize database connection pool
        info!("Initializing database connection pool...");
        let database_context = Arc::new(DatabaseContext::new().await?);

        // Initialize repositories
        info!("Initializing repositories...");
        let repositories = AppRepositories {
            podcast: PodcastRepository::new(database_context.clone()),
            podcast_rank: PodcastRankRepository::new(database_context.clone()),
            episode: EpisodeRepository::new(database_context.clone()),
        };
        let repositories = Arc::new(repositories);

        info!("Application initialization complete!");

        Ok(Self {
            repositories,
            database_context,
            settings: Arc::new(settings),
        })
    }

    /// Initialize the application state with default settings
    pub async fn init() -> AppResult<Self> {
        let settings = Settings::new()?;
        Self::init_with_settings(settings).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_app_initialization() {
        // Create test settings
        let mut settings = Settings::default();
        settings.database.url = "postgres://podcast:podcast@localhost/podcast_test".to_string(); // 使用测试数据库
        settings.database.max_connections = 2; // 限制最大连接数
        settings.logging.level = "debug".to_string(); // 设置日志级别

        // Attempt to initialize the app state
        let state_result = AppState::init_with_settings(settings).await;

        // Ensure the initialization does not fail
        assert!(state_result.is_ok(), "App initialization failed");

        // Extract the state for further verification
        let state = state_result.unwrap();

        // Verify database context is initialized
        assert!(
            Arc::strong_count(&state.database_context) > 1,
            "Database context not properly initialized"
        );

        // Verify repositories are initialized
        assert!(
            Arc::strong_count(&state.repositories) > 0,
            "Repositories not properly initialized"
        );
        assert!(
            state.repositories.podcast.get_all().await.is_ok(),
            "Podcast repository is not initialized"
        );
        assert!(
            state.repositories.podcast_rank.get_all().await.is_ok(),
            "Podcast rank repository is not initialized"
        );
        assert!(
            state.repositories.episode.get_all().await.is_ok(),
            "Episode repository is not initialized"
        );

        // Verify settings are correctly passed
        assert_eq!(state.settings.database.url, "postgres://podcast:podcast@localhost/podcast_test");
        assert_eq!(state.settings.logging.level, "debug");
    }
}
