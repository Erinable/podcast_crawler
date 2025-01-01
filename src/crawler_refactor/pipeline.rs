use crate::infrastructure::error::AppError;
use async_trait::async_trait;

#[async_trait]
pub trait Parser<T>: std::fmt::Debug {
    /// 解析feed内容为目标类型
    async fn parse(&self, content: &[u8], url: &str) -> Result<T, AppError>;
    async fn parse_with_task(
        &self,
        task: &mut crate::crawler_refactor::task::Task,
    ) -> Result<T, AppError>;
}

#[async_trait]
pub trait Fetcher: std::fmt::Debug {
    /// 获取内容
    async fn fetch(&self, url: &str) -> Result<Vec<u8>, AppError>;
    async fn fetch_with_task(
        &self,
        task: &mut crate::crawler_refactor::task::Task,
    ) -> Result<(), AppError>;
}
