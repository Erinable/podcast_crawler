use crate::infrastructure::error::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub connect_timeout_seconds: u64,
    pub idle_timeout_seconds: Option<u64>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub workers: usize,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CrawlerConfig {
    pub max_concurrent_tasks: usize,
    pub fetch_interval_seconds: u64,
    pub user_agent: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LoggingConfig {
    pub level: String,
    pub file_path: Option<String>,
    pub json_format: bool,
}

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
            database: DatabaseConfig {
                url: "postgres://localhost/podcast_crawler".to_string(),
                max_connections: 10,
                min_connections: 2,
                connect_timeout_seconds: 30,
                idle_timeout_seconds: Some(300),
            },
            server: ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 8080,
                workers: num_cpus::get(),
            },
            crawler: CrawlerConfig {
                max_concurrent_tasks: 5,
                fetch_interval_seconds: 3600,
                user_agent: "PodcastCrawler/1.0".to_string(),
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                file_path: Some("logs".to_string()),
                json_format: false,
            },
            is_test: false,
        }
    }
}

impl Settings {
    pub fn new() -> AppResult<Self> {
        // Load .env file if present
        dotenv::dotenv().ok();

        let mut settings = Self::default();

        // Database settings
        if let Ok(test_flag) = env::var("TEST_FLAG") {
            settings.is_test = test_flag.eq_ignore_ascii_case("true");
        }

        let database_url_key = if settings.is_test { "TEST_DATABASE_URL" } else { "DATABASE_URL" };
        if let Ok(url) = env::var(database_url_key) {
            settings.database.url = url;
        }
        if let Ok(max_conn) = env::var("DATABASE_MAX_CONNECTIONS") {
            settings.database.max_connections = max_conn
                .parse()
                .map_err(|_| AppError::config("Invalid DATABASE_MAX_CONNECTIONS"))?;
        }
        if let Ok(min_conn) = env::var("DATABASE_MIN_CONNECTIONS") {
            settings.database.min_connections = min_conn
                .parse()
                .map_err(|_| AppError::config("Invalid DATABASE_MIN_CONNECTIONS"))?;
        }
        if let Ok(timeout) = env::var("DATABASE_CONNECT_TIMEOUT") {
            settings.database.connect_timeout_seconds = timeout
                .parse()
                .map_err(|_| AppError::config("Invalid DATABASE_CONNECT_TIMEOUT"))?;
        }

        // Server settings
        if let Ok(host) = env::var("SERVER_HOST") {
            settings.server.host = host;
        }
        if let Ok(port) = env::var("SERVER_PORT") {
            settings.server.port = port
                .parse()
                .map_err(|_| AppError::config("Invalid SERVER_PORT"))?;
        }
        if let Ok(workers) = env::var("SERVER_WORKERS") {
            settings.server.workers = workers
                .parse()
                .map_err(|_| AppError::config("Invalid SERVER_WORKERS"))?;
        }

        // Crawler settings
        if let Ok(tasks) = env::var("CRAWLER_MAX_TASKS") {
            settings.crawler.max_concurrent_tasks = tasks
                .parse()
                .map_err(|_| AppError::config("Invalid CRAWLER_MAX_TASKS"))?;
        }
        if let Ok(interval) = env::var("CRAWLER_FETCH_INTERVAL") {
            settings.crawler.fetch_interval_seconds = interval
                .parse()
                .map_err(|_| AppError::config("Invalid CRAWLER_FETCH_INTERVAL"))?;
        }
        if let Ok(agent) = env::var("CRAWLER_USER_AGENT") {
            settings.crawler.user_agent = agent;
        }

        // Logging settings
        if let Ok(level) = env::var("LOG_LEVEL") {
            settings.logging.level = level;
        }
        if let Ok(path) = env::var("LOG_FILE") {
            settings.logging.file_path = Some(path);
        }
        if let Ok(json) = env::var("LOG_JSON") {
            settings.logging.json_format = json
                .parse()
                .map_err(|_| AppError::config("Invalid LOG_JSON"))?;
        }

        settings.validate()?;
        Ok(settings)
    }

    pub fn validate(&self) -> AppResult<()> {
        // Validate database settings
        if self.database.url.is_empty() {
            return Err(AppError::config("Database URL cannot be empty"));
        }
        if self.database.max_connections < self.database.min_connections {
            return Err(AppError::config(
                "Max connections must be >= min connections",
            ));
        }

        // Validate server settings
        if self.server.port == 0 {
            return Err(AppError::config("Server port cannot be 0"));
        }
        if self.server.workers == 0 {
            return Err(AppError::config("Server workers cannot be 0"));
        }

        // Validate crawler settings
        if self.crawler.max_concurrent_tasks == 0 {
            return Err(AppError::config("Max concurrent tasks cannot be 0"));
        }
        if self.crawler.fetch_interval_seconds == 0 {
            return Err(AppError::config("Fetch interval cannot be 0"));
        }

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
mod tests {
    use super::*;

    #[test]
    fn test_default_settings() {
        let settings = Settings::default();
        assert!(settings.validate().is_ok());
    }

    #[test]
    fn test_env_override() {
        env::set_var("DATABASE_URL", "postgres://test:5432/testdb");
        env::set_var("SERVER_PORT", "9090");
        env::set_var("LOG_LEVEL", "debug");

        let settings = Settings::new().unwrap();
        assert_eq!(settings.database_url(), "postgres://test:5432/testdb");
        assert_eq!(settings.server.port, 9090);
        assert_eq!(settings.log_level(), "debug");
    }
}
