use crate::infrastructure::AppResult;
use crate::{config_set_env, config_set_string, config_validate};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub workers: usize,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 8080,
            workers: 4,
        }
    }
}

impl ServerConfig {
    pub fn set_from_env(&mut self) -> AppResult<()> {
        config_set_string!(self, "SERVER_HOST", self.host);
        config_set_env!(self, "SERVER_PORT", self.port);
        config_set_env!(self, "SERVER_WORKERS", self.workers);
        Ok(())
    }

    pub fn validate(&self) -> AppResult<()> {
        config_validate!(!self.host.is_empty(), "Server host cannot be empty");
        config_validate!(self.port > 0, "Server port cannot be 0");
        config_validate!(self.workers > 0, "Server workers cannot be 0");
        Ok(())
    }
}
