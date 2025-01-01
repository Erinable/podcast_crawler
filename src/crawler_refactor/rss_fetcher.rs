use crate::crawler_refactor::pipeline::Fetcher;
use crate::infrastructure::error::{AppError, NetworkError, NetworkErrorKind};
use async_trait::async_trait;
use reqwest::Client;
use std::time::Duration;

#[derive(Clone, Debug)]
pub struct RssFetcher {
    client: Client,
    retry_delay: Duration,
}

#[async_trait]
impl Fetcher for RssFetcher {
    async fn fetch(&self, url: &str) -> Result<Vec<u8>, AppError> {
        let response = self
            .client
            .get(url)
            .header("Accept", "application/xml")
            .header("User-Agent", "PodcastCrawler/1.0")
            .send()
            .await
            .map_err(|e| {
                NetworkError::new(
                    NetworkErrorKind::Connection,
                    e.to_string(),
                    None,
                    Some(Box::new(e)),
                )
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let headers = response.headers().clone();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "No error text".to_string());
            return Err(AppError::Network(NetworkError::new(
                NetworkErrorKind::InvalidResponse,
                format!(
                    "HTTP request failed with status: {}, headers: {:?}, body: {}",
                    status, headers, error_text
                ),
                None,
                None,
            )));
        }

        let bytes = response
            .bytes()
            .await
            .map_err(|e| {
                NetworkError::new(
                    NetworkErrorKind::Connection,
                    e.to_string(),
                    None,
                    Some(Box::new(e)),
                )
            })?
            .to_vec();

        Ok(bytes)
    }

    async fn fetch_with_task(
        &self,
        task: &mut crate::crawler_refactor::task::Task,
    ) -> Result<(), AppError> {
        let url = task.payload.clone();

        // 如果 task 没有 fetching 阶段，则添加
        if !task.stages.iter().any(|s| s.name == "fetching") {
            task.add_stage("fetching");
        }

        // 执行 fetch，失败时直接返回错误，外部逻辑会处理 fail_stage
        let data = self.fetch(&url).await?;
        task.content = data;
        task.complete_stage(serde_json::json!({}));
        Ok(())
    }
}

impl RssFetcher {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .tcp_nodelay(true)
            .pool_max_idle_per_host(0)
            .no_proxy()
            .build()
            .expect("Failed to create HTTP client");
        Self {
            client,
            retry_delay: Duration::from_secs(1),
        }
    }
}
