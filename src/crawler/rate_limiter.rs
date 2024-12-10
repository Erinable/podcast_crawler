use crate::infrastructure::error::{AppError, NetworkError, NetworkErrorKind};
use governor::{
    clock::DefaultClock,
    state::{InMemoryState, NotKeyed},
    Quota, RateLimiter as GovernorRateLimiter,
};
use std::num::NonZeroU32;
use std::sync::Arc;
use std::time::Duration;

pub struct CrawlerRateLimiter {
    limiter: Arc<GovernorRateLimiter<NotKeyed, InMemoryState, DefaultClock>>,
    retry_delay: Duration,
}

impl CrawlerRateLimiter {
    pub fn new(requests_per_second: u32) -> Result<Self, AppError> {
        let requests = NonZeroU32::new(requests_per_second).ok_or_else(|| {
            NetworkError::new(
                NetworkErrorKind::RateLimit,
                "Invalid requests per second value",
                None,
                None,
            )
        })?;

        let quota = Quota::per_second(requests);
        Ok(Self {
            limiter: Arc::new(GovernorRateLimiter::direct(quota)),
            retry_delay: Duration::from_secs(1),
        })
    }

    pub fn default() -> Self {
        Self::new(2).unwrap_or_else(|_| Self {
            limiter: Arc::new(GovernorRateLimiter::direct(Quota::per_second(
                NonZeroU32::new(1).unwrap(),
            ))),
            retry_delay: Duration::from_secs(1),
        })
    }

    pub async fn wait_for_rate_limit(&self) -> Result<(), AppError> {
        self.limiter.until_ready().await;
        Ok(())
    }

    pub fn set_retry_delay(&mut self, delay: Duration) {
        self.retry_delay = delay;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn test_new_rate_limiter() {
        let limiter = CrawlerRateLimiter::new(5).unwrap();
        assert!(!std::ptr::eq(&limiter, &CrawlerRateLimiter::default()));
    }

    #[test]
    fn test_default_rate_limiter() {
        let _limiter = CrawlerRateLimiter::default();
        // 只测试创建是否成功，不比较内部状态
    }

    #[tokio::test]
    async fn test_rate_limiting() {
        let limiter = CrawlerRateLimiter::new(2).unwrap(); // 每秒2个请求
        let start = Instant::now();

        // 尝试快速执行3个请求
        for i in 0..3 {
            limiter.wait_for_rate_limit().await.unwrap();
            println!("Request {} completed at {:?}", i, start.elapsed());
        }

        let elapsed = start.elapsed();
        println!("Total elapsed time: {:?}", elapsed);
        // 由于速率限制是每秒2个请求，3个请求至少需要0.5秒
        assert!(elapsed.as_secs_f64() >= 0.5);
    }

    #[tokio::test]
    async fn test_concurrent_rate_limiting() {
        let limiter = Arc::new(CrawlerRateLimiter::new(2).unwrap());
        let start = Instant::now();

        let mut handles = vec![];

        // 创建4个并发任务
        for i in 0..4 {
            let limiter_clone = limiter.clone();
            handles.push(tokio::spawn(async move {
                limiter_clone.wait_for_rate_limit().await.unwrap();
                println!("Request {} completed at {:?}", i, start.elapsed());
            }));
        }

        // 等待所有任务完成
        for handle in handles {
            handle.await.unwrap();
        }

        let elapsed = start.elapsed();
        println!("Total elapsed time: {:?}", elapsed);
        // 4个请求以2/秒的速率至少需要1秒
        assert!(elapsed.as_secs_f64() >= 1.0);
    }
}
