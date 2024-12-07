use crate::infrastructure::config::AppResult;
use crate::{config_set_env, config_set_string, config_validate};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CrawlerConfig {
    pub max_concurrent_tasks: usize,
    pub fetch_interval_seconds: u64,
    pub user_agent: String,
}

impl Default for CrawlerConfig {
    fn default() -> Self {
        Self {
            max_concurrent_tasks: 5,
            fetch_interval_seconds: 3600,
            user_agent: "PodcastCrawler/1.0".to_string(),
        }
    }
}

impl CrawlerConfig {
    pub fn set_from_env(&mut self) -> AppResult<()> {
        config_set_string!(self, "CRAWLER_USER_AGENT", self.user_agent);
        config_set_env!(self, "CRAWLER_MAX_TASKS", self.max_concurrent_tasks);
        config_set_env!(self, "CRAWLER_FETCH_INTERVAL", self.fetch_interval_seconds);
        Ok(())
    }

    pub fn validate(&self) -> AppResult<()> {
        config_validate!(
            self.max_concurrent_tasks > 0,
            "Max concurrent tasks must be > 0"
        );
        config_validate!(
            self.fetch_interval_seconds > 0,
            "Fetch interval must be > 0"
        );
        config_validate!(!self.user_agent.is_empty(), "User agent cannot be empty");
        Ok(())
    }
}
