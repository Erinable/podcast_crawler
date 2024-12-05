use thiserror::Error;
use diesel::result::Error as DieselError;
use diesel_async::pooled_connection::PoolError;
use bb8::RunError;
use reqwest::Error as ReqwestError;
use std::io::Error as IoError;
use tracing::{error, instrument};

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] DieselError),

    #[error("Connection pool error: {0}")]
    Pool(#[from] RunError<PoolError>),

    #[error("IO error: {0}")]
    Io(#[from] IoError),

    #[error("HTTP error: {0}")]
    Http(#[from] ReqwestError),

    #[error("Logging initialization error: {0}")]
    LoggingInit(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Other error: {0}")]
    Other(String),
}

pub type AppResult<T> = Result<T, AppError>;

impl AppError {
    // 记录错误信息和上下文
    pub fn log_context(&self, custom_message: impl Into<String>) {
        let message = custom_message.into();
        error!("{}: {:?}", message, self); // 记录错误和自定义消息
    }

    // Helper functions for common error cases
    #[instrument]
    pub fn not_found(msg: impl Into<String> + std::fmt::Debug) -> Self {
        AppError::NotFound(msg.into())
    }

    #[instrument]
    pub fn validation(msg: impl Into<String> + std::fmt::Debug) -> Self {
        AppError::Validation(msg.into())
    }

    #[instrument]
    pub fn config(msg: impl Into<String> + std::fmt::Debug) -> Self {
        AppError::Config(msg.into())
    }

    #[instrument]
    pub fn database(msg: impl Into<String> + std::fmt::Debug) -> Self {
        AppError::Database(DieselError::DatabaseError(
            diesel::result::DatabaseErrorKind::Unknown,
            Box::new(msg.into()),
        ))
    }
}

// 扩展 Result 以支持自动转换和记录
pub trait AppResultExt<T> {
    fn error_info(self, msg: impl Into<String>) -> Result<T, AppError>;
}

impl<T, E> AppResultExt<T> for Result<T, E>
where
    E: Into<AppError>,
{
    fn error_info(self, msg: impl Into<String>) -> Result<T, AppError> {
        self.map_err(|e| {
            let error = e.into();
            error.log_context(msg); // 记录上下文信息
            error
        })
    }
}


