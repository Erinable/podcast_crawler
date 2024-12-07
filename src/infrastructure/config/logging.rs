use crate::infrastructure::config::AppResult;
use crate::{config_set_env, config_set_string, config_validate};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LoggingConfig {
    pub level: String,
    pub file_path: String,
    pub json_format: bool,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            file_path: "logs".to_string(),
            json_format: false,
        }
    }
}

impl LoggingConfig {
    pub fn set_from_env(&mut self) -> AppResult<()> {
        config_set_string!(self, "LOG_LEVEL", self.level);
        config_set_string!(self, "LOG_FILE", self.file_path);
        config_set_env!(self, "LOG_JSON", self.json_format);
        self.validate()?;
        Ok(())
    }

    pub fn validate(&self) -> AppResult<()> {
        config_validate!(!self.level.is_empty(), "Log level cannot be empty");
        config_validate!(
            matches!(
                self.level.as_str(),
                "error" | "warn" | "info" | "debug" | "trace"
            ),
            "Invalid log level. Must be one of: error, warn, info, debug, trace"
        );
        config_validate!(!self.file_path.is_empty(), "Log file path cannot be empty");
        Ok(())
    }
}
