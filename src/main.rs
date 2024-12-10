use tracing::info;

use podcast_crawler::crawler::{rss::RssFeedParser, HttpCrawler};
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

    // Create URLs for the mock server
    let urls = state.repositories.podcast_rank.get_rss_urls().await?;
    // Initialize the RSS parser and HTTP crawler
    let parser = RssFeedParser::new();
    let crawler = HttpCrawler::new(parser, 10);
    // Perform the crawl
    let results = try_with_log!(crawler.crawl_batch(urls[..10].to_vec()).await);
    info!("Results: {:#?}", results);

    Ok(())
}
