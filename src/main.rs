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

    // Mount XML resources from the tests/data directory
    let xml_feeds = ["complex_feed.xml", "xiaoyuzhou.xml"];

    // Create URLs for the mock server
    let mut urls = xml_feeds
        .iter()
        .enumerate()
        .map(|(i, _)| format!("{}/feed{}", "http://localhost/facke_server", i + 1))
        .collect::<Vec<_>>();

    info!("Generated URLs: {:?}", urls);
    urls.push("http://www.ximalaya.com/album/20527677.xml".to_string());
    // Initialize the RSS parser and HTTP crawler
    let parser = RssFeedParser::new();
    let crawler = HttpCrawler::new(parser, 2);
    // Perform the crawl
    let results = try_with_log!(crawler.crawl_batch(urls.clone()).await);
    info!("Results: {:#?}", results);

    Ok(())
}
