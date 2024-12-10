use crate::infrastructure::error::{AppError, try_with_log, try_with_warn, try_with_debug, try_with_retry};
use std::io;
use tokio;
use tracing;
use std::time::Duration;
use crate::infrastructure::error::external::{ExternalError, ExternalErrorKind};

// 一个可能失败的函数
fn fetch_podcast_data(id: &str) -> Result<String, io::Error> {
    Err(io::Error::new(io::ErrorKind::NotFound, "Podcast not found"))
}

// 展示不同的错误处理方式
pub fn example_error_handling(podcast_id: &str) -> Result<(), AppError> {
    // 1. 基本用法 - 只记录错误
    try_with_log!(fetch_podcast_data("123"));

    // 2. 带上下文的用法 - 添加错误发生的上下文信息
    try_with_warn!(
        fetch_podcast_data("456"),
        "Failed to fetch podcast data during sync"
    );

    // 3. 带上下文和额外字段的用法 - 添加更多调试信息
    try_with_debug!(
        fetch_podcast_data(podcast_id),
        "Failed to fetch podcast data",
        "podcast_id" = podcast_id,
        "retry_count" = 3
    );

    Ok(())
}

// 展示重试逻辑的例子
pub async fn fetch_with_retry(url: &str) -> Result<String, AppError> {
    // 使用 try_with_retry 宏，支持多种用法

    // 1. 基本用法 - 使用默认重试次数
    try_with_retry!(fetch_url(url)).await?;

    // 2. 自定义重试次数
    try_with_retry!(fetch_url(url), max_attempts = 5).await?;

    // 3. 带上下文和额外字段的完整用法
    try_with_retry!(
        fetch_url(url),
        max_attempts = 3,
        context = "Failed to fetch podcast URL",
        "url" = url,
        "service" = "podcast_api"
    ).await
}

// 模拟的 HTTP 请求函数
async fn fetch_url(url: &str) -> Result<String, AppError> {
    // 模拟一个限流错误
    Err(AppError::External(ExternalError {
        kind: ExternalErrorKind::RateLimit,
        message: "Too many requests".into(),
        retry_after: Some(Duration::from_secs(5)),
        error_code: "E_RATE_LIMIT".into(),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_handling() {
        let result = example_error_handling("test-id");
        assert!(result.is_err());
    }
}
