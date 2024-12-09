use std::time::Duration;
use std::time::Instant;

use tracing::info;

use crate::crawler::traits::Crawler;
use crate::infrastructure::error::{AppError, DomainError, DomainErrorKind};

#[derive(Debug)]
pub enum TaskResult<T> {
    Success { 
        data: T, 
        duration: Duration,
        batch_index: usize,
        max_batches: usize 
    },
    Failure { 
        error: AppError, 
        url: String,
        batch_index: usize,
        max_batches: usize,
        duration: Duration 
    },
}

impl<T> TaskResult<T> {
    pub fn success(data: T, duration: Duration, batch_index: usize, max_batches: usize) -> Self {
        TaskResult::Success { 
            data, 
            duration, 
            batch_index, 
            max_batches 
        }
    }

    pub fn failure(
        error: AppError, 
        url: String, 
        batch_index: usize, 
        max_batches: usize,
        duration: Duration
    ) -> Self {
        TaskResult::Failure { 
            error, 
            url, 
            batch_index, 
            max_batches,
            duration 
        }
    }

    pub fn is_success(&self) -> bool {
        matches!(self, TaskResult::Success { .. })
    }

    pub fn parsed_data(&self) -> Option<&T> {
        match self {
            TaskResult::Success { data, .. } => Some(data),
            _ => None
        }
    }
}

pub async fn process_batch<T: Clone + Send + 'static>(
    crawler: impl Crawler<T> + Clone + Send + Sync + 'static,
    urls: &[String],
    insert_batch: usize,
    batch_index: usize,
    max_batches: usize,
    insert_fn: impl Fn(Vec<T>) -> Result<(), AppError> + Send + Sync + 'static,
) -> Result<Vec<TaskResult<T>>, AppError> {
    let start_time = Instant::now();

    let handles: Vec<_> = urls.iter()
        .map(|url| {
            let url = url.clone();
            let crawler = crawler.clone();
            tokio::spawn(async move {
                let task_start = Instant::now();
                match crawler.fetch_and_parse(&url).await {
                    Ok(result) => TaskResult::success(
                        result, 
                        task_start.elapsed(), 
                        batch_index, 
                        max_batches
                    ),
                    Err(e) => TaskResult::failure(
                        e, 
                        url, 
                        batch_index, 
                        max_batches,
                        task_start.elapsed()
                    )
                }
            })
        })
        .collect::<Vec<_>>();

    let results: Vec<TaskResult<T>> = futures::future::try_join_all(
        handles.into_iter().map(|handle| async move {
            match handle.await {
                Ok(task_result) => Ok::<TaskResult<T>, AppError>(task_result),
                Err(join_error) => Ok(TaskResult::failure(
                    AppError::from(DomainError::new(
                        DomainErrorKind::Unexpected,
                        format!("Task join error: {}", join_error),
                        None,
                        None,
                    )),
                    "unknown".to_string(),
                    batch_index,
                    max_batches,
                    Duration::default()
                ))
            }
        })
    )
    .await?;

    let successful_results: Vec<T> = results
        .iter()
        .filter_map(|result| result.parsed_data().cloned())
        .collect();

    if !successful_results.is_empty() {
        insert_fn(successful_results)?;
    }

    let (success, failure) = count_results(&results);
    log_batch_completion(start_time, success, failure);

    Ok(results)
}

fn log_batch_completion(start_time: Instant, success: usize, failure: usize) {
    info!(
        success_count = success,
        failure_count = failure,
        total_duration = ?start_time.elapsed(),
        "Batch processing completed"
    );
}

fn count_results<T>(results: &[TaskResult<T>]) -> (usize, usize) {
    results.iter().fold((0, 0), |(success, failure), result| {
        if result.is_success() {
            (success + 1, failure)
        } else {
            (success, failure + 1)
        }
    })
}
