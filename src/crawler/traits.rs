use crate::infrastructure::error::AppError;
use async_trait::async_trait;

#[async_trait]
pub trait Crawler<T>: Send + Sync + Clone
where
    T: Send + 'static,
{
    /// 抓取单个URL的内容
    async fn fetch(&self, url: &str) -> Result<Vec<u8>, AppError>;

    /// 解析内容
    async fn parse(&self, content: Vec<u8>, url: &str) -> Result<T, AppError>;

    /// 抓取并解析内容
    async fn fetch_and_parse(&self, url: &str) -> Result<T, AppError> {
        let content = self.fetch(url).await?;
        self.parse(content, url).await
    }

    /// 获取最大并发数
    fn max_concurrent(&self) -> usize;
}

#[async_trait]
pub trait FeedParser<T> {
    /// 解析feed内容为目标类型
    async fn parse(&self, content: &[u8], url: &str) -> Result<T, AppError>;
}
