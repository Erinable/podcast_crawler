use std::future::Future;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::task;
use std::time::Duration;

use tokio::sync::{mpsc, Mutex, Semaphore};
use tokio::task::JoinHandle;
use tokio::time::timeout;
use tracing::{error, info, warn};

use super::task::Task;

#[derive(Clone, Debug)]
pub struct BatchInserter {
    tx: mpsc::Sender<Task>,
    rx: Arc<Mutex<mpsc::Receiver<Task>>>,
    processed_count: Arc<AtomicUsize>,
    semaphore: Arc<Semaphore>,
    active_workers: Arc<AtomicUsize>,
    monitor_handle: Arc<Mutex<Option<JoinHandle<Result<(), String>>>>>,
    monitor_shutdown: Arc<Mutex<Option<mpsc::Sender<()>>>>,
}

impl BatchInserter {
    pub fn new<F, Fut>(
        batch_size: usize,
        max_concurrent_inserts: usize,
        insert_fn: F,
        batch_timeout: Duration,
    ) -> Self
    where
        F: Fn(Vec<Task>) -> Fut + Send + Sync + 'static + Clone,
        Fut: Future<Output = Result<(), String>> + Send,
    {
        let (tx, rx) = mpsc::channel(5000);
        let rx = Arc::new(Mutex::new(rx));

        let processed_count = Arc::new(AtomicUsize::new(0));
        let semaphore = Arc::new(Semaphore::new(max_concurrent_inserts));
        let active_workers = Arc::new(AtomicUsize::new(0));
        let (monitor_shutdown_tx, monitor_shutdown_rx) = mpsc::channel(1);
        let monitor_shutdown = Arc::new(Mutex::new(Some(monitor_shutdown_tx)));

        let monitor_handle = Self::spawn_monitor(
            rx.clone(),
            batch_size,
            batch_timeout,
            insert_fn,
            processed_count.clone(),
            semaphore.clone(),
            active_workers.clone(),
            monitor_shutdown_rx, // Pass the receiver here!
        );

        Self {
            tx,
            rx,
            processed_count,
            semaphore,
            active_workers,
            monitor_handle: Arc::new(Mutex::new(Some(monitor_handle))),
            monitor_shutdown,
        }
    }

    fn spawn_monitor<F, Fut>(
        rx: Arc<Mutex<mpsc::Receiver<Task>>>,
        batch_size: usize,
        batch_timeout: Duration,
        insert_fn: F,
        processed_count: Arc<AtomicUsize>,
        semaphore: Arc<Semaphore>,
        active_workers: Arc<AtomicUsize>,
        mut monitor_shutdown_rx: mpsc::Receiver<()>, // Take ownership of the receiver
    ) -> JoinHandle<Result<(), String>>
    where
        F: Fn(Vec<Task>) -> Fut + Send + Sync + 'static + Clone,
        Fut: Future<Output = Result<(), String>> + Send,
    {
        tokio::spawn(async move {
            loop {
                // Check for shutdown signal
                if monitor_shutdown_rx.try_recv().is_ok() {
                    info!(
                        "Monitor received shutdown signal, waiting for active workers to complete"
                    );
                    while active_workers.load(Ordering::SeqCst) > 0 {
                        tokio::time::sleep(Duration::from_millis(100)).await;
                    }
                    info!("Monitor exiting");
                    return Ok(());
                }

                let batch_result =
                    timeout(batch_timeout, Self::collect_batch(rx.clone(), batch_size)).await;

                let batch = match batch_result {
                    Ok(batch) => {
                        info!("Batch collection completed with {} items", batch.len());
                        batch
                    }
                    Err(_) => {
                        // warn!("Batch timeout reached with no items.");
                        continue;
                    }
                };

                if batch.is_empty() {
                    info!("No items to process; continue waiting data.");
                    continue; // channel is closed or shutdown signal received
                }

                info!("Collected batch of {} items.", batch.len());

                let semaphore = semaphore.clone();
                let processed_count = processed_count.clone();
                let active_workers = active_workers.clone();
                let insert_fn = insert_fn.clone();

                active_workers.fetch_add(1, Ordering::Relaxed);

                tokio::spawn(async move {
                    let _permit = semaphore.acquire().await;
                    if let Err(e) = insert_fn(batch).await {
                        error!("Error processing batch: {:?}", e);
                        // we could implement retries here
                    } else {
                        processed_count.fetch_add(1, Ordering::Relaxed);
                    }
                    active_workers.fetch_sub(1, Ordering::Relaxed);
                });
            }
        })
    }

    async fn collect_batch(rx: Arc<Mutex<mpsc::Receiver<Task>>>, batch_size: usize) -> Vec<Task> {
        let mut batch = Vec::with_capacity(batch_size);
        let mut rx = rx.lock().await;

        while batch.len() < batch_size {
            match timeout(Duration::from_millis(500), rx.recv()).await {
                Ok(Some(item)) => {
                    info!(
                        "Received task id: {:?},batch len: {:?},batch size: {:?}",
                        item.id,
                        batch.len(),
                        batch_size
                    );
                    batch.push(item)
                }
                Ok(None) => {
                    warn!("Channel closed, stopping collection.");
                    break; // return empty if the channel closes
                }
                Err(_) => {
                    if batch.is_empty() {
                        // warn!("No items received during timeout.");
                        continue;
                    }
                    break; // return a partial batch in the case of a timeout
                }
            }
        }

        info!("Returning batch with {} items", batch.len());
        batch
    }

    pub async fn insert(&self, task: Task) -> Result<(), mpsc::error::SendError<Task>> {
        info!("inserter send task id: {:?}", task.id);
        let result = self.tx.send(task).await;
        if result.is_ok() {
            info!("Task successfully sent to channel");
        } else {
            warn!("Failed to send task to channel");
        }
        result
    }

    pub async fn finish(self) -> Result<usize, String> {
        // Signal shutdown to the monitor thread
        let monitor_shutdown_tx = self.monitor_shutdown.lock().await.take();
        if let Some(tx) = monitor_shutdown_tx {
            if let Err(_e) = tx.send(()).await {
                error!("Error sending shutdown signal to the monitor thread");
            }
        }

        drop(self.tx); // close the channel
        let handle = self.monitor_handle.lock().await.take();

        if let Some(handle) = handle {
            match timeout(Duration::from_secs(10), handle).await {
                Ok(Ok(_)) => {
                    info!("Monitor thread finished successfully");
                }
                Ok(Err(e)) => {
                    error!("Monitor thread returned error: {:?}", e);
                    return Err(format!("Monitor thread returned error: {:?}", e));
                }
                Err(e) => {
                    error!("Error awaiting monitor handle: {:?}", e);
                    return Err(format!("Error awaiting monitor handle: {:?}", e));
                }
            }
        }

        while self.active_workers.load(Ordering::SeqCst) > 0 {
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        info!("BatchInserter finished processing all items.");
        Ok(self.processed_count.load(Ordering::SeqCst))
    }
}
