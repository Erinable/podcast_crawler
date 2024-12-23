use std::future::Future;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::sync::{mpsc, Mutex, Semaphore};
use tokio::task::JoinHandle;
use tokio::time::timeout;
use tracing::{error, info, warn};

use futures::stream::{Stream, StreamExt};
use rand::Rng;
use std::collections::BinaryHeap;

// Placeholder types (replace with your actual types)
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct InputData {
    value: u32,
    retry_count: u32,
    last_attempt_time: std::time::Instant,
}

#[derive(Clone, Debug)]
struct Data(u32);

// Wrapper for BinaryHeap to implement min-heap
#[derive(PartialEq, Eq, Debug)]
struct PriorityItem<T: Ord + PartialOrd> {
    priority: u64,
    item: T,
}

impl<T: Ord + PartialOrd> Ord for PriorityItem<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Reverse ordering for min-heap
        other.priority.cmp(&self.priority)
        // other.item.partial_cmp(&self.item).unwrap_or(Ordering::Equal) // You can also compare the items here
    }
}

impl<T: Ord + PartialOrd> PartialOrd for PriorityItem<T> {
    fn partial_cmp(&self, other: &Self) -> std::option::Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

fn calculate_backoff(retry_count: u32) -> Duration {
    let base_delay_ms = 10;
    let jitter_ms = rand::thread_rng().gen_range(0..100);
    let delay_ms = base_delay_ms * 2u64.pow(retry_count) + jitter_ms;

    Duration::from_millis(delay_ms)
}

async fn generate_data(
    input_data: InputData,
    inserter: BatchInserterHandle<Data>,
    mistake_tx: mpsc::Sender<InputData>,
    max_retries: u32,
) -> Result<(), ()> {
    // Simulate data generation with potential error
    if input_data.value % 5 != 0 {
        info!("Generating data from input {:?}", input_data);
        let data = Data(input_data.value * 2);
        inserter.insert(data).await.map_err(|_| ())?; // Send Data
        Ok(())
    } else {
        if input_data.retry_count < max_retries {
            error!("Error generating data for input {:?}", input_data);
            let mut input_data = input_data.clone();
            input_data.retry_count += 1;
            input_data.last_attempt_time = std::time::Instant::now();
            mistake_tx.send(input_data.clone()).await.map_err(|_| ())?;
            info!("Data added to PQ {:?}", input_data);
        } else {
            error!(
                "Error generating data for input {:?} - Dropping Data",
                input_data
            );
            info!("Dropping data {:?}", input_data);
        }

        Err(())
    }
}

async fn distribute_data(
    mut data_stream: impl Stream<Item = InputData> + Unpin,
    inserter: BatchInserterHandle<Data>,
    num_generators: usize,
    max_retries: u32,
) {
    let (mistake_tx, mut mistake_rx) = mpsc::channel::<InputData>(100);
    let mut handles = Vec::new();
    let mut priority_queue: BinaryHeap<PriorityItem<InputData>> = BinaryHeap::new();

    while let Some(input) = data_stream.next().await {
        priority_queue.push(PriorityItem {
            priority: input.last_attempt_time.elapsed().as_millis() as u64,
            item: input,
        })
    }

    let mut current_generator = 0;

    while !priority_queue.is_empty() {
        while let Ok(data) = mistake_rx.try_recv() {
            info!("Data received from mistake channel {:?}", data);
            let backoff_duration = calculate_backoff(data.retry_count);
            tokio::time::sleep(backoff_duration).await;
            priority_queue.push(PriorityItem {
                priority: data.last_attempt_time.elapsed().as_millis() as u64,
                item: data,
            })
        }

        let num_items = std::cmp::min(priority_queue.len(), num_generators);
        for _ in 0..num_items {
            if let Some(PriorityItem {
                item: input_data, ..
            }) = priority_queue.pop()
            {
                let inserter_clone = inserter.clone();
                let mistake_tx_clone = mistake_tx.clone();
                let handle = tokio::spawn(async move {
                    let _result =
                        generate_data(input_data, inserter_clone, mistake_tx_clone, max_retries)
                            .await;
                });
                handles.push(handle);

                current_generator = (current_generator + 1) % num_generators;
            }
        }

        tokio::time::sleep(Duration::from_millis(10)).await;
    }

    drop(mistake_tx); // close the mistake queue
    while let Ok(data) = mistake_rx.try_recv() {
        warn!("Unprocessed Data in Mistake Queue {:?}", data);
    }
    for handle in handles {
        handle.await.unwrap();
    }

    info!("Distributor finished");
}

async fn generate_input_stream(num_items: u32) -> impl Stream<Item = InputData> {
    tokio_stream::iter((0..num_items).map(|value| InputData {
        value,
        retry_count: 0,
        last_attempt_time: std::time::Instant::now(),
    }))
}

#[derive(Clone)]
pub struct BatchInserter<T> {
    tx: mpsc::Sender<T>,
    rx: Arc<Mutex<mpsc::Receiver<T>>>,
    processed_count: Arc<AtomicUsize>,
    semaphore: Arc<Semaphore>,
    active_workers: Arc<AtomicUsize>,
    monitor_handle: Arc<Mutex<Option<JoinHandle<Result<(), String>>>>>,
    monitor_shutdown: Arc<Mutex<Option<mpsc::Sender<()>>>>,
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
    ) -> Self
    where
        F: Fn(Vec<T>) -> Fut + Send + Sync + 'static + Clone,
        Fut: Future<Output = Result<(), String>> + Send,
    {
        let (tx, rx) = mpsc::channel(batch_size * 2);
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
        rx: Arc<Mutex<mpsc::Receiver<T>>>,
        batch_size: usize,
        batch_timeout: Duration,
        insert_fn: F,
        processed_count: Arc<AtomicUsize>,
        semaphore: Arc<Semaphore>,
        active_workers: Arc<AtomicUsize>,
        mut monitor_shutdown_rx: mpsc::Receiver<()>, // Take ownership of the receiver
    ) -> JoinHandle<Result<(), String>>
    where
        F: Fn(Vec<T>) -> Fut + Send + Sync + 'static + Clone,
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

                let batch =
                    match timeout(batch_timeout, Self::collect_batch(rx.clone(), batch_size)).await
                    {
                        Ok(batch) => batch,
                        Err(_) => {
                            warn!("Batch timeout reached with no items.");
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
            while active_workers.load(Ordering::SeqCst) > 0 {
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
            info!("Monitor exiting");
            Ok(())
        })
    }

    async fn collect_batch(rx: Arc<Mutex<mpsc::Receiver<T>>>, batch_size: usize) -> Vec<T> {
        let mut batch = Vec::with_capacity(batch_size);
        let mut rx = rx.lock().await;

        while batch.len() < batch_size {
            match timeout(Duration::from_millis(5), rx.recv()).await {
                Ok(Some(item)) => batch.push(item),
                Ok(None) => {
                    warn!("Channel closed, stopping collection.");
                    break; // return empty if the channel closes
                }
                Err(_) => {
                    if batch.is_empty() {
                        warn!("No items received during timeout.");
                    }
                    continue; // return a partial batch in the case of a timeout
                }
            }
        }

        batch
    }

    pub async fn insert(&self, item: T) -> Result<(), mpsc::error::SendError<T>> {
        self.tx.send(item).await
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

// New struct to wrap BatchInserter and only expose the insert method
pub struct BatchInserterHandle<T> {
    inserter: Arc<BatchInserter<T>>,
}

impl<T> Clone for BatchInserterHandle<T>
where
    T: Send + 'static + Clone,
{
    fn clone(&self) -> Self {
        Self {
            inserter: self.inserter.clone(),
        }
    }
}

impl<T> BatchInserterHandle<T>
where
    T: Send + 'static + Clone,
{
    pub fn new(inserter: BatchInserter<T>) -> Self {
        Self {
            inserter: Arc::new(inserter),
        }
    }

    pub async fn insert(&self, item: T) -> Result<(), mpsc::error::SendError<T>> {
        self.inserter.insert(item).await
    }
}

#[tokio::test]
async fn test_batch_inserter() {
    tracing_subscriber::fmt::init();

    let batch_size = 5;
    let max_concurrent_inserts = 3;
    let batch_timeout = Duration::from_millis(5000);

    let inserter = BatchInserter::new(
        batch_size,
        max_concurrent_inserts,
        |batch: Vec<u32>| async move {
            info!("Inserting batch: {:?}", batch);
            Ok(())
        },
        batch_timeout,
    );

    for i in 0..13 {
        inserter.insert(i).await.unwrap();
    }

    let final_count = inserter.finish().await.unwrap();

    assert_eq!(15 / batch_size, final_count);
}

#[tokio::test]
async fn test_batch_inserter_multithreaded() {
    tracing_subscriber::fmt::init();

    let batch_size = 5;
    let max_concurrent_inserts = 1;
    let batch_timeout = Duration::from_millis(500);

    let inserter = BatchInserter::new(
        batch_size,
        max_concurrent_inserts,
        |batch: Vec<u32>| async move {
            info!("Inserting batch: {:?}", batch);
            Ok(())
        },
        batch_timeout,
    );

    // Number of items to insert and number of threads
    let num_items = 50;
    let num_threads = 4;

    let mut handles = vec![];

    let inserter_handler = BatchInserterHandle::new(inserter.clone());

    // Simulate multi-threaded inserts
    for thread_id in 0..num_threads {
        let inserter_clone = inserter_handler.clone();
        let start = thread_id * (num_items / num_threads);
        let end = start + (num_items / num_threads);

        handles.push(tokio::spawn(async move {
            for i in start..end {
                inserter_clone.insert(i as u32).await.unwrap();
                info!("Thread {} inserted {}", thread_id, i);
            }
        }));
    }

    // Wait for all threads to finish
    for handle in handles {
        handle.await.unwrap();
    }

    // Finish the BatchInserter
    let final_count = inserter.finish().await.unwrap();

    assert_eq!(num_items / batch_size, final_count);
}

#[tokio::test]
async fn test_distributer_with_inserter() {
    tracing_subscriber::fmt::init();
    let inserter = BatchInserter::new(
        50, // batch_size
        10, // max_concurrent_inserts
        |batch: Vec<Data>| async move {
            info!("Inserting batch: {:?}", batch);
            Ok(())
        },
        Duration::from_millis(5000),
    );

    let inserter_handle = BatchInserterHandle::new(inserter.clone());

    let input_stream = generate_input_stream(200).await;

    distribute_data(input_stream, inserter_handle, 20, 1).await;

    let final_count = inserter.finish().await;
    println!("Processed {} items", final_count.unwrap());
}
