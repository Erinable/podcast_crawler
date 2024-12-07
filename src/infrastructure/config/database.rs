use crate::infrastructure::config::utils;
use crate::infrastructure::config::AppResult;
use crate::{config_set_env, config_validate};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub connect_timeout_seconds: u64,
    pub idle_timeout_seconds: u64,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: "postgres://localhost:5432/podcast".to_string(),
            max_connections: 10,
            min_connections: 2,
            connect_timeout_seconds: 30,
            idle_timeout_seconds: 300,
        }
    }
}

impl DatabaseConfig {
    pub fn set_from_env(&mut self, is_test: bool) -> AppResult<()> {
        // Special handling for database URL in test mode
        self.url = utils::get_env_string_with_test("DATABASE_URL", "TEST_DATABASE_URL", is_test)
            .unwrap_or(self.url.clone());

        // Other database settings
        config_set_env!(self, "DATABASE_MAX_CONNECTIONS", self.max_connections);
        config_set_env!(self, "DATABASE_MIN_CONNECTIONS", self.min_connections);
        config_set_env!(
            self,
            "DATABASE_CONNECT_TIMEOUT",
            self.connect_timeout_seconds
        );
        config_set_env!(self, "DATABASE_IDLE_TIMEOUT", self.idle_timeout_seconds);
        Ok(())
    }

    pub fn validate(&self) -> AppResult<()> {
        config_validate!(!self.url.is_empty(), "Database URL cannot be empty");
        config_validate!(
            self.max_connections >= self.min_connections,
            "Max connections must be >= min connections"
        );
        config_validate!(
            self.connect_timeout_seconds > 0,
            "Connect timeout must be > 0"
        );
        config_validate!(self.idle_timeout_seconds > 0, "Idle timeout must be > 0");
        Ok(())
    }
}
