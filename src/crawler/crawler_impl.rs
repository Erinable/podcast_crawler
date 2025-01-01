use crate::crawler::batch_processor;
use crate::crawler::traits::Crawler;
use crate::{
    infrastructure::error::{
        AppError, AppResult, ExternalErrorKind, NetworkError, NetworkErrorKind,
    },
    try_with_retry,
};
use std::collections::HashMap;
use std::marker::PhantomData;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::sync::Semaphore;
use tokio::time;
use tracing::{error, info};

use super::TaskResult;

pub struct HttpCrawler<P, T>
where
    P: super::traits::FeedParser<T> + Send + Sync + 'static + Clone,
    T: Send + Sync + 'static + Clone,
{
    client: reqwest::Client,
    parser: Arc<P>,
    concurrent_limit: Arc<Semaphore>,
    max_concurrent: usize,
    retry_delay: Duration,
    _marker: PhantomData<T>,
    failed_tasks: Arc<AtomicUsize>,
    successful_tasks: Arc<AtomicUsize>,
    max_retries: usize,
    total_time: Arc<Mutex<Duration>>,
    failure_reasons: Arc<Mutex<Vec<String>>>,
    total_tasks: Arc<AtomicUsize>,
}

impl<P, T> Clone for HttpCrawler<P, T>
where
    P: super::traits::FeedParser<T> + Send + Sync + 'static + Clone,
    T: Send + Sync + 'static + Clone,
{
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
            parser: Arc::clone(&self.parser),
            concurrent_limit: Arc::clone(&self.concurrent_limit),
            max_concurrent: self.max_concurrent,
            retry_delay: self.retry_delay,
            _marker: PhantomData,
            failed_tasks: Arc::clone(&self.failed_tasks),
            successful_tasks: Arc::clone(&self.successful_tasks),
            max_retries: self.max_retries,
            total_time: Arc::clone(&self.total_time),
            failure_reasons: Arc::clone(&self.failure_reasons),
            total_tasks: Arc::clone(&self.total_tasks),
        }
    }
}

impl<P, T> HttpCrawler<P, T>
where
    P: super::traits::FeedParser<T> + Send + Sync + 'static + Clone,
    T: Send + Sync + 'static + Clone,
{
    pub fn new(parser: P, max_concurrent: usize) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .tcp_nodelay(true) // 禁用 Nagle 算法，减少延迟
            .pool_max_idle_per_host(0) // 避免连接池闲置阻塞
            .no_proxy() // 禁用代理
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            parser: Arc::new(parser),
            concurrent_limit: Arc::new(Semaphore::new(max_concurrent)),
            max_concurrent,
            max_retries: 3,
            retry_delay: Duration::from_secs(1),
            _marker: std::marker::PhantomData,
            failed_tasks: Arc::new(AtomicUsize::new(0)),
            successful_tasks: Arc::new(AtomicUsize::new(0)),
            total_time: Arc::new(Mutex::new(Duration::new(0, 0))),
            failure_reasons: Arc::new(Mutex::new(Vec::new())),
            total_tasks: Arc::new(AtomicUsize::new(0)),
        }
    }

    pub fn with_retry_config(mut self, max_retries: usize, retry_delay: Duration) -> Self {
        self.max_retries = max_retries;
        self.retry_delay = retry_delay;
        self
    }

    async fn fetch_internal(&self, url: &str) -> Result<Vec<u8>, AppError> {
        try_with_retry!(
            {
                let _permit = self.concurrent_limit.acquire().await.map_err(|e| {
                    NetworkError::new(
                        NetworkErrorKind::RateLimit,
                        format!("Failed to acquire semaphore: {}", e),
                        Some(self.retry_delay),
                        Some(Box::new(e)),
                    )
                })?;

                let response = self.client.get(url).send().await.map_err(|e| {
                    NetworkError::new(
                        NetworkErrorKind::Connection,
                        format!("Failed to send request to {}: {}", url, e),
                        Some(self.retry_delay),
                        Some(Box::new(e)),
                    )
                })?;

                if !response.status().is_success() {
                    return Err(AppError::Network(NetworkError::new(
                        NetworkErrorKind::InvalidResponse,
                        format!("HTTP request failed with status {}", response.status()),
                        Some(self.retry_delay),
                        None,
                    )));
                }

                response.bytes().await.map(|b| b.to_vec()).map_err(|e| {
                    NetworkError::new(
                        NetworkErrorKind::Connection,
                        format!("Failed to read response body from {}: {}", url, e),
                        Some(self.retry_delay),
                        Some(Box::new(e)),
                    )
                })
            },
            max_attempts = 3,
            context = "Failed to fetch URL"
        )
    }

    async fn fetch_once(&self, url: &str) -> Result<Vec<u8>, AppError> {
        let response = self.client.get(url).send().await.map_err(|e| {
            NetworkError::new(
                NetworkErrorKind::Connection,
                format!("Failed to send request: {}", e),
                Some(self.retry_delay),
                Some(Box::new(e)),
            )
        })?;

        if !response.status().is_success() {
            return Err(AppError::Network(NetworkError::new(
                NetworkErrorKind::InvalidResponse,
                format!("HTTP request failed with status: {}", response.status()),
                Some(self.retry_delay),
                None,
            )));
        }

        response.bytes().await.map(|b| b.to_vec()).map_err(|e| {
            AppError::Network(NetworkError::new(
                NetworkErrorKind::Connection,
                format!("Failed to read response body: {}", e),
                Some(self.retry_delay),
                Some(Box::new(e)),
            ))
        })
    }

    pub async fn crawl_batch(&self, urls: Vec<String>) -> Result<Vec<TaskResult<T>>, AppError> {
        batch_processor::run_batch_processor(self, urls).await
    }

    pub async fn crawl_batch_with_inserter<F, D>(
        &mut self,
        urls: Vec<String>,
        insert_batch: usize,
        insert_fn: F,
    ) -> Result<Vec<TaskResult<T>>, AppError>
    where
        F: Fn(Vec<D>) -> Result<(), AppError> + Send + Sync + 'static + Clone,
        D: Send + 'static + From<T> + Into<T>,
        T: Clone,
    {
        batch_processor::run_batch_processor_with_inserter(
            self,
            urls,
            insert_batch,
            move |batch: Vec<T>| {
                let converted_batch: Vec<D> = batch.into_iter().map(|item| item.into()).collect();
                insert_fn(converted_batch)
            },
        )
        .await
    }

    // async fn fetch_and_parse(&self, url: &str) -> TaskResult<T> {
    //     let start = Instant::now();
    //     let result = match self.fetch(url).await {
    //         Ok(result) => match self.parse(result, url).await {
    //             Ok(parsed) => TaskResult::success(url.to_string(), parsed, start.elapsed()),
    //             Err(e) => TaskResult::failure(url.to_string(), e.to_string(), start.elapsed()),
    //         },
    //         Err(e) => TaskResult::failure(url.to_string(), e.to_string(), start.elapsed()),
    //     };
    //     result
    // }

    // pub fn max_concurrent(&self) -> usize {
    //     self.max_concurrent
    // }

    // pub fn report_statistics(&self) {
    //     let success_rate = if self.total_tasks.load(Ordering::SeqCst) > 0 {
    //         (self.successful_tasks.load(Ordering::SeqCst) as f64
    //             / self.total_tasks.load(Ordering::SeqCst) as f64)
    //             * 100.0
    //     } else {
    //         0.0
    //     };

    //     let avg_time = if self.total_tasks.load(Ordering::SeqCst) > 0 {
    //         let total_time_locked = self.total_time.lock().unwrap();
    //         let total_tasks_count = self.total_tasks.load(Ordering::SeqCst);
    //         total_time_locked.div_f64(total_tasks_count as f64)
    //     } else {
    //         Duration::new(0, 0)
    //     };

    //     println!("\n=== Crawler Statistics ===");
    //     println!("Total tasks: {}", self.total_tasks.load(Ordering::SeqCst));
    //     println!(
    //         "Successful tasks: {} ({:.1}%)",
    //         self.successful_tasks.load(Ordering::SeqCst),
    //         success_rate
    //     );
    //     println!("Failed tasks: {}", self.failed_tasks.load(Ordering::SeqCst));
    //     println!("Average completion time: {:?}", avg_time);
    //     println!("\nFailure Reasons:");
    //     for reason in self.failure_reasons.lock().unwrap().iter() {
    //         println!("  {}", reason);
    //     }
    //     println!("=====================");
    // }
}

#[async_trait::async_trait]
impl<P, T> Crawler<T> for HttpCrawler<P, T>
where
    P: super::traits::FeedParser<T> + Send + Sync + 'static + Clone,
    T: Send + Sync + 'static + Clone,
{
    async fn fetch(&self, url: &str) -> Result<Vec<u8>, AppError> {
        info!("Attempting to fetch URL: {}", url);
        let response = self
            .client
            .get(url)
            .header("Accept", "application/xml")
            .header("User-Agent", "PodcastCrawler/1.0")
            .send()
            .await
            .map_err(|e| {
                println!("Connection error: {}", e);
                NetworkError::new(
                    NetworkErrorKind::Connection,
                    e.to_string(),
                    None,
                    Some(Box::new(e)),
                )
            })?;

        info!("Response status: {}", response.status());
        info!("Response headers: {:?}", response.headers());

        if !response.status().is_success() {
            let status = response.status();
            let headers = response.headers().clone();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "No error text".to_string());
            println!("Response body: {}", error_text);
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
                println!("Bytes read error: {}", e);
                NetworkError::new(
                    NetworkErrorKind::Connection,
                    e.to_string(),
                    None,
                    Some(Box::new(e)),
                )
            })?
            .to_vec();

        info!("Bytes read successfully: {} bytes", bytes.len());
        Ok(bytes)
    }

    async fn parse(&self, content: Vec<u8>, url: &str) -> Result<T, AppError> {
        let parser = self.parser.clone();
        parser.parse(&content, url).await
    }

    // async fn fetch_and_parse(&self, url: &str) -> Result<T, AppError> {
    //     let result = match self.fetch(url).await {
    //         Ok(result) => match self.parse(result, url).await {
    //             Ok(parsed) => parsed,
    //             Err(e) => return Err(e),
    //         },
    //         Err(e) => return Err(e),
    //     };

    //     Ok(result)
    // }

    fn max_concurrent(&self) -> usize {
        self.max_concurrent
    }
}
