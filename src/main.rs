use tracing::info;

use podcast_crawler::{
    infrastructure::{initialize, AppResult},
    try_with_log,
};

#[tokio::main]
async fn main() -> AppResult<()> {
    let state = initialize().await?;

    // 测试错误处理宏
    try_with_log!(state.health_check().await, "Health check completed");

    info!("Application started successfully");

    Ok(())
}
