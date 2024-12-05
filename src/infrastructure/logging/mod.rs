use std::path::Path;
use tracing_appender::rolling::{InitError, RollingFileAppender, Rotation};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use crate::infrastructure::config::LoggingConfig;
use crate::infrastructure::error::{AppError, AppResult};

impl From<InitError> for AppError {
    fn from(err: InitError) -> Self {
        AppError::LoggingInit(err.to_string())
    }
}

pub fn init_logger(config: &LoggingConfig) -> AppResult<()> {
    let file_path = match &config.file_path {
        Some(path) => Path::new(path),
        None => Path::new("logs"),
    };
    std::fs::create_dir_all(file_path)?;

    let file_appender = RollingFileAppender::builder()
        .rotation(Rotation::DAILY)
        .filename_prefix("podcast_crawler")
        .filename_suffix("log")
        .build(file_path)?;

    let file_layer = fmt::layer()
        .with_writer(file_appender)
        .with_ansi(false)
        .with_target(true)
        .with_thread_ids(true)
        .with_thread_names(true);

    let stdout_layer = fmt::layer()
        .with_writer(std::io::stdout)
        .with_ansi(true)
        .with_target(false)
        .with_thread_ids(false)
        .with_thread_names(false);

    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(file_layer)
        .with(stdout_layer)
        .init();

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tracing::info;

    #[test]
    fn test_init_logger() {
        let config = LoggingConfig {
            level: "debug".to_string(),
            file_path: Some("test_logs".to_string()),
            json_format: false,
        };
        init_logger(&config).unwrap();
        info!("Test log message");
    }
}
