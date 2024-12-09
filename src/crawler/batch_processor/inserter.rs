use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{error, info};

use crate::infrastructure::error::{AppError, DomainError, DomainErrorKind};

pub(crate) struct BatchInserter<T> {
    tx: mpsc::Sender<T>,
    processed_count: Arc<AtomicUsize>,
    total_count: usize,
}

impl<T> BatchInserter<T>
where
    T: Send + 'static,
{
    pub fn new<F>(batch_size: usize, insert_fn: F) -> Self
    where
        F: Fn(Vec<T>) -> Result<(), AppError> + Send + Sync + 'static,
    {
        let (tx, mut rx) = mpsc::channel(batch_size * 2);
        let processed_count = Arc::new(AtomicUsize::new(0));
        let counter = processed_count.clone();

        tokio::spawn(async move {
            let mut current_batch = Vec::with_capacity(batch_size);

            while let Some(item) = rx.recv().await {
                current_batch.push(item);

                if current_batch.len() >= batch_size {
                    Self::process_batch(&insert_fn, &mut current_batch, &counter).await;
                }
            }

            // Process remaining items
            if !current_batch.is_empty() {
                Self::process_batch(&insert_fn, &mut current_batch, &counter).await;
            }
        });

        Self {
            tx,
            processed_count,
            total_count: 0,
        }
    }

    pub async fn insert(&self, item: T) {
        if let Err(e) = self.tx.send(item).await {
            error!(
                error = %e,
                "Failed to send item to batch inserter queue",
            );
        }
    }

    pub async fn finish(self) {
        drop(self.tx);
    }

    async fn process_batch<F>(insert_fn: &F, batch: &mut Vec<T>, counter: &Arc<AtomicUsize>)
    where
        F: Fn(Vec<T>) -> Result<(), AppError> + Send + Sync,
    {
        match insert_fn(std::mem::take(batch)) {
            Ok(_) => {
                counter.fetch_add(batch.len(), Ordering::SeqCst);
                info!("Successfully processed batch of {} items", batch.len());
            }
            Err(e) => {
                error!("Failed to process batch: {}", e);
                let error = DomainError::new(
                    DomainErrorKind::BatchProcessing,
                    format!("Batch processing error: {}", e),
                    Some(format!("Batch size: {}", batch.len())),
                    Some(Box::new(e)),
                );
                error!("Batch error: {}", error);
            }
        }
    }
}
