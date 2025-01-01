use std::collections::HashMap;
use std::future::Future;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::mpsc::error::TryRecvError;
use tokio::sync::{mpsc, Mutex, Semaphore};
use tokio::task::JoinHandle;
use tracing::{error, info};

use crate::crawler::batch_processor::processor::process_batch;
use crate::crawler::batch_processor::stats::{BatchStats, StatsSummary};
use crate::crawler::TaskResult;
use crate::infrastructure::error::{AppError, DomainError, DomainErrorKind};

pub(crate) struct BatchInserter<T> {
    tx: mpsc::Sender<T>,
    rx: Arc<Mutex<mpsc::Receiver<T>>>,
    processed_count: Arc<AtomicUsize>,
    semaphore: Arc<Semaphore>,
    active_workers: Arc<AtomicUsize>,
    channel_capacity: usize,
    total_count: usize,
    stats: Arc<BatchStats>,
}

impl<T> BatchInserter<T>
where
    T: Send + 'static + Clone,
{
    pub fn new<F, Fut>(
        batch_size: usize,
        max_concurrent_inserts: usize,
        insert_fn: F,
        batch_timeout: Duration,
        concurrent_threads: usize,
    ) -> Self
    where
        F: Fn(Vec<T>) -> Fut + Send + Sync + 'static + Clone,
        Fut: Future<Output = Result<(), AppError>> + Send,
    {
        println!(
            "Creating BatchInserter: batch_size={}, max_concurrent_inserts={}, batch_timeout={:?}, concurrent_threads={}",
            batch_size, max_concurrent_inserts, batch_timeout, concurrent_threads
        );

        // Calculate optimal channel capacity
        let channel_capacity = batch_size * concurrent_threads;
        let (tx, rx) = mpsc::channel(channel_capacity);
        let rx = Arc::new(Mutex::new(rx));
        let processed_count = Arc::new(AtomicUsize::new(0));
        let semaphore = Arc::new(Semaphore::new(max_concurrent_inserts));
        let active_workers = Arc::new(AtomicUsize::new(0));
        let stats = Arc::new(BatchStats::new());

        let processed_count_clone = processed_count.clone();
        let semaphore_clone = semaphore.clone();
        let active_workers_clone = active_workers.clone();
        let stats_clone = stats.clone();
        let tx_clone = tx.clone();
        let rx_clone = rx.clone();

        tokio::spawn(async move {
            let this = Self {
                tx: tx_clone,
                rx: rx_clone,
                processed_count: processed_count_clone.clone(),
                semaphore: semaphore_clone.clone(),
                active_workers: active_workers_clone.clone(),
                total_count: 0,
                stats: stats_clone.clone(),
                channel_capacity,
            };
            this.start_channel_monitor(
                batch_size,
                batch_timeout,
                insert_fn.clone(),
                processed_count_clone,
                semaphore_clone,
                active_workers_clone,
                stats_clone,
            )
            .await
            .await
        });

        Self {
            tx,
            rx,
            processed_count,
            semaphore,
            active_workers,
            total_count: 0,
            stats,
            channel_capacity,
        }
    }

    async fn start_channel_monitor<F, Fut>(
        &self,
        batch_size: usize,
        batch_timeout: Duration,
        insert_fn: F,
        processed_count: Arc<AtomicUsize>,
        semaphore: Arc<Semaphore>,
        active_workers: Arc<AtomicUsize>,
        stats: Arc<BatchStats>,
    ) -> JoinHandle<()>
    where
        F: Fn(Vec<T>) -> Fut + Send + Sync + 'static + Clone,
        Fut: Future<Output = Result<(), AppError>> + Send,
    {
        let rx_clone = self.rx.clone();
        tokio::spawn(async move {
            BatchInserter::channel_monitor(
                rx_clone,
                batch_size,
                batch_timeout,
                insert_fn,
                processed_count,
                semaphore,
                active_workers,
                stats,
            )
            .await
        })
    }

    async fn channel_monitor<F, Fut>(
        rx: Arc<Mutex<mpsc::Receiver<T>>>,
        batch_size: usize,
        batch_timeout: Duration,
        insert_fn: F,
        processed_count: Arc<AtomicUsize>,
        semaphore: Arc<Semaphore>,
        active_workers: Arc<AtomicUsize>,
        stats: Arc<BatchStats>,
    ) where
        F: Fn(Vec<T>) -> Fut + Send + Sync + 'static + Clone,
        Fut: Future<Output = Result<(), AppError>> + Send,
    {
        loop {
            // Collect a batch of items
            let batch = Self::collect_batch(&rx, batch_size, batch_timeout).await;

            // Skip if batch is empty
            if batch.is_empty() {
                tokio::time::sleep(Duration::from_millis(100)).await;
                continue;
            }

            // Spawn a worker to process the batch
            let worker_batch = batch.clone();
            let worker_insert_fn = insert_fn.clone();
            let worker_processed_count = processed_count.clone();
            let worker_semaphore = semaphore.clone();
            let worker_active_workers = active_workers.clone();
            let worker_stats = stats.clone();

            tokio::spawn(async move {
                Self::spawn_worker(
                    worker_batch,
                    worker_insert_fn,
                    worker_processed_count,
                    worker_semaphore,
                    worker_active_workers,
                    worker_stats,
                )
                .await;
            });
        }
    }

    async fn collect_batch(
        rx: &Arc<Mutex<mpsc::Receiver<T>>>,
        batch_size: usize,
        batch_timeout: Duration,
    ) -> Vec<T> {
        let mut batch = Vec::with_capacity(batch_size);
        let start_time = Instant::now();

        // Try to fill the batch
        while batch.len() < batch_size {
            // Check timeout
            if start_time.elapsed() >= batch_timeout {
                break;
            }

            // Use a timeout for receiving to allow checking the elapsed time
            let timeout_duration = Duration::from_millis(50);
            let receive_result =
                tokio::time::timeout(timeout_duration, rx.lock().await.recv()).await;

            match receive_result {
                Ok(Some(item)) => {
                    batch.push(item);
                }
                Ok(None) => {
                    // Channel closed
                    break;
                }
                Err(_) => {
                    // Timeout occurred, continue checking
                    continue;
                }
            }
        }

        batch
    }

    async fn spawn_worker<F, Fut>(
        batch: Vec<T>,
        insert_fn: F,
        counter: Arc<AtomicUsize>,
        semaphore: Arc<Semaphore>,
        active_workers: Arc<AtomicUsize>,
        stats: Arc<BatchStats>,
    ) where
        F: Fn(Vec<T>) -> Fut + Send + Sync + 'static + Clone,
        Fut: Future<Output = Result<(), AppError>> + Send,
    {
        println!(
            "Spawning worker for batch of {} items. Active workers before spawn: {}",
            batch.len(),
            active_workers.load(Ordering::SeqCst)
        );

        active_workers.fetch_add(1, Ordering::SeqCst);

        tokio::spawn(async move {
            BatchInserter::worker_thread(
                batch,
                &insert_fn,
                &counter,
                &semaphore,
                &active_workers,
                &stats,
            )
            .await;
        });
    }

    async fn worker_thread<F, Fut>(
        batch: Vec<T>,
        insert_fn: &F,
        _counter: &Arc<AtomicUsize>,
        semaphore: &Arc<Semaphore>,
        active_workers: &Arc<AtomicUsize>,
        stats: &Arc<BatchStats>,
    ) where
        F: Fn(Vec<T>) -> Fut + Send + Sync + 'static + Clone,
        Fut: Future<Output = Result<(), AppError>> + Send,
    {
        println!(
            "Worker thread started with batch of {} items. Active workers: {}",
            batch.len(),
            active_workers.load(Ordering::SeqCst)
        );

        let _permit = semaphore.acquire().await.unwrap();
        let start_time = Instant::now();
        let batch_size = batch.len();

        println!("Attempting to process batch of {} items", batch_size);

        let task_result = match insert_fn(batch).await {
            Ok(_) => {
                println!("Batch processing successful");
                TaskResult::success("".to_string(), batch_size, start_time.elapsed())
            }
            Err(e) => {
                println!("Batch processing failed: {}", e);
                TaskResult::failure("error".to_string(), e.to_string(), start_time.elapsed())
            }
        };

        stats.record_result(&task_result);

        println!(
            "Worker thread completed. Batch size: {}, Duration: {:?}, Active workers before decrement: {}",
            batch_size,
            start_time.elapsed(),
            active_workers.load(Ordering::SeqCst)
        );

        active_workers.fetch_sub(1, Ordering::SeqCst);
    }

    pub async fn insert(&self, item: T) {
        println!("Attempting to insert item into batch inserter queue");
        if let Err(e) = self.tx.send(item).await {
            error!(
                error = %e,
                "Failed to send item to batch inserter queue",
            );
        }
    }

    pub async fn finish(self) {
        println!("Finishing batch inserter: Dropping sender to signal channel closure");
        drop(self.tx);
    }

    pub async fn get_stats_summary(&self) -> StatsSummary {
        let summary = self.stats.get_summary().await;
        println!("Stats Summary:\n {}", summary.format_report());
        summary
    }

    async fn process_batch<F, Fut>(insert_fn: &F, batch: &mut Vec<T>, counter: &Arc<AtomicUsize>)
    where
        F: Fn(Vec<T>) -> Fut + Send + Sync + 'static + Clone,
        Fut: Future<Output = Result<(), AppError>> + Send,
    {
        println!("Processing batch of {} items", batch.len());
        match insert_fn(std::mem::take(batch)).await {
            Ok(_) => {
                counter.fetch_add(batch.len(), Ordering::SeqCst);
                info!("Successfully processed batch of {} items", batch.len());
                println!(
                    "Batch processed successfully. Total processed: {}",
                    counter.load(Ordering::SeqCst)
                );
            }
            Err(e) => {
                error!("Failed to process batch: {}", e);
                println!("Batch processing failed with error: {}", e);
                let error = DomainError::new(
                    DomainErrorKind::BatchProcessing,
                    format!("Batch processing error: {}", e),
                    None,
                    Some(Box::new(e)),
                );
                error!("Batch error: {}", error);
                println!("Detailed batch error: {}", error);
            }
        }
    }
}
