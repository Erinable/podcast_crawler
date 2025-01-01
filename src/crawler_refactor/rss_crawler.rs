use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use tracing::{error, info};

use crate::infrastructure::AppState;

use super::{task::Task, task_management_system::TaskManagementSystem};

/// RSS爬虫系统入口
pub struct RssCrawler {
    system: TaskManagementSystem,
}

impl RssCrawler {
    /// 创建新的RSS爬虫实例
    ///
    /// # 参数
    /// - worker_count: 工作线程数量
    /// - max_history_size: 每个worker最大历史任务记录
    pub async fn new(state: Arc<AppState>, worker_count: usize, max_history_size: usize) -> Self {
        let system = TaskManagementSystem::new(state, worker_count, max_history_size).await;
        Self { system }
    }

    /// 启动爬虫系统
    pub async fn start(&mut self) {
        self.system.start().await;
    }

    /// 添加新的爬取任务
    ///
    /// # 参数
    /// - url: 要爬取的RSS feed URL
    pub async fn add_task(&mut self, url: &str) -> Result<(), String> {
        let start = Instant::now();

        let result = self.system.add_task(url).await;
        let duration = start.elapsed().as_secs_f64();

        match &result {
            Ok(_) => {
                info!("✅ Task completed in {:.2}s", duration);
            }
            Err(e) => {
                error!("❌ Task failed: {}", e);
            }
        }

        result
    }

    /// 获取所有任务状态
    pub async fn get_tasks(&self) -> Vec<Task> {
        self.system.get_task_info().await
    }

    /// 等待所有任务完成
    ///
    /// # 返回
    /// 所有已完成的任务列表
    pub async fn wait_for_completion(&self) -> Vec<Task> {
        self.system.wait_for_all_tasks_completed().await
    }

    /// 优雅关闭爬虫系统
    pub async fn shutdown(&self) {
        self.system.shutdown().await;
    }

    /// 带超时的优雅关闭
    ///
    /// # 参数
    /// - timeout: 关闭超时时间
    pub async fn shutdown_with_timeout(&self, timeout: Duration) {
        self.system.shutdown_with_timeout(timeout).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::initialize;
    #[tokio::test]
    async fn test_rss_crawler() {
        let state = initialize().await.unwrap();
        let mut crawler = RssCrawler::new(Arc::new(state), 2, 10).await;
        crawler.start().await;

        // 添加测试任务
        crawler
            .add_task("http://example.com/feed.rss")
            .await
            .unwrap();

        // 等待任务完成
        let tasks = crawler.wait_for_completion().await;
        assert!(!tasks.is_empty());

        // 关闭系统
        crawler.shutdown().await;
    }
}
