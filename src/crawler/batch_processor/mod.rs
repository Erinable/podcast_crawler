mod processor;
mod inserter;
mod stats;

use crate::crawler::traits::Crawler;
use crate::crawler::TaskResult;
use crate::crawler::url_utils;
use crate::infrastructure::error::{AppError, DomainError, DomainErrorKind};

pub(crate) use processor::process_batch;
pub(crate) use inserter::BatchInserter;

impl<T> From<processor::TaskResult<T>> for TaskResult<T> {
    fn from(result: processor::TaskResult<T>) -> Self {
        match result {
            processor::TaskResult::Success { data, duration, batch_index: _, max_batches: _ } => {
                TaskResult {
                    url: "".to_string(), // Note: loss of original URL
                    success: true,
                    parsed_data: Some(data),
                    error_message: None,
                    duration,
                }
            },
            processor::TaskResult::Failure { error, url, batch_index: _, max_batches: _, duration } => {
                TaskResult {
                    url,
                    success: false,
                    parsed_data: None,
                    error_message: Some(error.to_string()),
                    duration,
                }
            }
        }
    }
}

impl<T> From<TaskResult<T>> for processor::TaskResult<T> {
    fn from(result: TaskResult<T>) -> Self {
        if result.success {
            processor::TaskResult::Success {
                data: result.parsed_data.unwrap(),
                duration: result.duration,
                batch_index: 0, // Note: loss of original batch index
                max_batches: 0, // Note: loss of original max batches
            }
        } else {
            processor::TaskResult::Failure {
                error: AppError::from(DomainError::new(
                    DomainErrorKind::Unexpected,
                    result.error_message.unwrap_or_default(),
                    None,
                    None,
                )),
                url: result.url,
                batch_index: 0, // Note: loss of original batch index
                max_batches: 0, // Note: loss of original max batches
                duration: result.duration,
            }
        }
    }
}

pub(crate) async fn run_batch_processor<T, P>(
    crawler: &P,
    urls: Vec<String>,
) -> Result<Vec<TaskResult<T>>, AppError>
where
    T: Send + 'static + Clone,
    P: Crawler<T> + Clone + Send + Sync + 'static,
{
    run_batch_processor_with_inserter(crawler, urls, 1, |_: Vec<T>| Ok(())).await
}

pub(crate) async fn run_batch_processor_with_inserter<T, P, F>(
    crawler: &P,
    urls: Vec<String>,
    insert_batch: usize,
    insert_fn: F,
) -> Result<Vec<TaskResult<T>>, AppError>
where
    T: Send + 'static + Clone,
    P: Crawler<T> + Clone + Send + Sync + 'static,
    F: Fn(Vec<T>) -> Result<(), AppError> + Send + Sync + 'static + Clone,
{
    if urls.is_empty() {
        return Ok(vec![]);
    }

    let mut results = vec![];
    let distributed_urls = url_utils::distribute_urls(&urls, insert_batch)?;

    for (batch_index, batch_urls) in distributed_urls.iter().enumerate() {
        let batch_results = processor::process_batch(
            crawler.clone(),
            batch_urls,
            insert_batch,
            batch_index,
            distributed_urls.len(),
            insert_fn.clone(),
        ).await?;
        results.extend(batch_results.into_iter().map(|r| r.into()).collect::<Vec<_>>());
    }

    Ok(results)
}
