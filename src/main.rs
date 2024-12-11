use tracing::info;

use podcast_crawler::crawler::{rss::RssFeedParser, HttpCrawler};
use podcast_crawler::{
    infrastructure::{initialize, AppResult},
    try_with_log,
};

use rand::seq::SliceRandom; // 用于从切片中随机选择
use rand::thread_rng; // 获取随机数生成器

#[tokio::main]
async fn main() -> AppResult<()> {
    let state = initialize().await?;

    // 定义要随机挑选的个数 n
    let n = 20;

    // 获取一个线程本地的随机数生成器
    let mut rng = thread_rng();

    // 测试错误处理宏
    try_with_log!(state.health_check().await, "Health check completed");

    info!("Application started successfully");

    // Create URLs for the mock server
    let urls = state.repositories.podcast_rank.get_rss_urls().await?;
    let random_samples: Vec<_> = urls.choose_multiple(&mut rng, n).cloned().collect();
    // Initialize the RSS parser and HTTP crawler
    let parser = RssFeedParser::new();
    let crawler = HttpCrawler::new(parser, 10);
    // Perform the crawl
    let results = try_with_log!(crawler.crawl_batch(random_samples).await);
    info!("Results: {:#?}", results);

    Ok(())
}
