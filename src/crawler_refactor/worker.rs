use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use tokio::sync::broadcast;
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info, warn};

use super::{
    task::Task,
    task_management_system::{ShutdownCoordinator, TaskWorkerMaps},
    timer_queue::TimerQueue,
};

use crate::infrastructure::error::{
    AppError, DomainError, DomainErrorKind, NetworkError, NetworkErrorKind,
};

/// Worker状态
#[derive(Debug, Clone, PartialEq)]
enum WorkerState {
    Idle,
    Processing,
    Draining,
    Shutdown,
}

/// Internal Worker structure
#[derive(Debug, Clone)]
pub struct Worker {
    pub id: usize,
    max_history_size: usize,
    state: WorkerState,
    task_worker_maps: Arc<TaskWorkerMaps>,
    metrics: WorkerMetrics,
}

#[derive(Debug, Clone)]
pub struct WorkerMetrics {
    tasks_processed: u64,
    tasks_failed: u64,
    tasks_retried: u64,
    avg_process_time: Duration,
}

impl Worker {
    pub fn new(id: usize, max_history_size: usize, task_worker_maps: Arc<TaskWorkerMaps>) -> Self {
        Self {
            id,
            max_history_size,
            state: WorkerState::Idle,
            task_worker_maps,
            metrics: WorkerMetrics {
                tasks_processed: 0,
                tasks_failed: 0,
                tasks_retried: 0,
                avg_process_time: Duration::ZERO,
            },
        }
    }

    pub async fn start(
        &mut self,
        mut worker_task_rx: broadcast::Receiver<Task>,
        worker_cancellation_token: CancellationToken,
        timer_queue: Arc<TimerQueue>,
        shutdown_coordinator: Arc<ShutdownCoordinator>,
    ) {
        info!(worker_id = self.id, "Starting worker");
        self.state = WorkerState::Processing;

        let mut in_progress_tasks = Vec::new();

        loop {
            tokio::select! {
                result = worker_task_rx.recv() => {
                    match result {
                        Ok(mut task) => self.handle_task(&mut task, &timer_queue, &mut in_progress_tasks).await,
                        Err(e) => {
                            warn!(worker_id = self.id, "Task channel error: {}", e);
                            continue;
                        }
                    }
                }
                _ = worker_cancellation_token.cancelled() => {
                    self.handle_shutdown(&shutdown_coordinator, &mut in_progress_tasks).await;
                    break;
                }
            }
        }
    }

    async fn handle_task(
        &mut self,
        task: &mut Task,
        timer_queue: &Arc<TimerQueue>,
        in_progress_tasks: &mut Vec<u64>,
    ) {
        if self.state != WorkerState::Processing || task.target_thread_id != self.id {
            // debug!(worker_id = self.id, "Skipping non-target task");
            return;
        }

        info!(worker_id = self.id, task_id = task.id, "Processing task");
        self.metrics.tasks_processed += 1;
        in_progress_tasks.push(task.id);

        let start_time = Instant::now();
        let result = self.process_task(task, timer_queue).await;
        let process_time = start_time.elapsed();

        self.update_metrics(process_time, result.is_err());
        in_progress_tasks.retain(|&id| id != task.id);

        if let Err(e) = result {
            error!(worker_id = self.id, task_id = task.id, "Task failed: {}", e);
        } else {
            info!(worker_id = self.id, task_id = task.id, "Task completed");
        }
    }

    async fn process_task(
        &mut self,
        task: &mut Task,
        timer_queue: &Arc<TimerQueue>,
    ) -> Result<(), AppError> {
        let fetch_result = self.fetch_task(task).await;
        if let Err(e) = fetch_result {
            return self.handle_fetch_error(task, timer_queue, e).await;
        }

        self.parse_task(task).await?;

        // Insert parsed data
        self.insert_task(task).await?;
        self.task_worker_maps
            .update_task(task.id, task.clone())
            .await;
        self.update_history(&task.payload).await;

        Ok(())
    }

    async fn handle_fetch_error(
        &mut self,
        task: &mut Task,
        timer_queue: &Arc<TimerQueue>,
        error: String,
    ) -> Result<(), AppError> {
        if task.retries < task.max_retries {
            self.metrics.tasks_retried += 1;
            task.retries += 1;
            task.backoff_timer = Some(Instant::now() + Duration::from_secs(1));
            timer_queue.schedule_retry(task.clone());
            return Err(AppError::Network(NetworkError::new(
                NetworkErrorKind::Connection,
                error,
                None,
                Some(Box::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!(
                        "Fetch failed, retrying (attempt {}/{})",
                        task.retries, task.max_retries
                    ),
                ))),
            )));
        }

        self.metrics.tasks_failed += 1;
        task.error_message = Some(error.clone());
        task.fail_stage(error.clone());
        self.task_worker_maps
            .update_task(task.id, task.clone())
            .await;

        Err(AppError::Network(NetworkError::new(
            NetworkErrorKind::Connection,
            error,
            None,
            Some(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Max retries ({}) reached", task.max_retries),
            ))),
        )))
    }

    async fn handle_shutdown(
        &mut self,
        shutdown_coordinator: &Arc<ShutdownCoordinator>,
        in_progress_tasks: &mut [u64],
    ) {
        self.state = WorkerState::Draining;
        info!(worker_id = self.id, "Initiating shutdown");

        // Wait for timer queue
        if !shutdown_coordinator.wait_for_timer_queue().await {
            warn!(worker_id = self.id, "Timer queue timeout during shutdown");
        }

        // Mark in-progress tasks as failed
        for &task_id in in_progress_tasks.iter() {
            if let Some(mut task) = self.task_worker_maps.read_task(&task_id).await {
                task.error_message = Some("Shutdown signal".to_string());
                task.fail_stage("Shutdown signal".to_string());
                self.task_worker_maps.update_task(task.id, task).await;
            }
        }

        self.state = WorkerState::Shutdown;
        shutdown_coordinator.worker_completed();
        info!(worker_id = self.id, "Shutdown completed");
    }

    fn update_metrics(&mut self, process_time: Duration, failed: bool) {
        if failed {
            self.metrics.tasks_failed += 1;
        }

        // Update average process time
        let total_time =
            self.metrics.avg_process_time * self.metrics.tasks_processed as u32 + process_time;
        self.metrics.avg_process_time = total_time / (self.metrics.tasks_processed + 1) as u32;
    }

    async fn fetch_task(&mut self, task: &mut Task) -> Result<(), String> {
        let fetcher = self.task_worker_maps.get_fetcher();
        fetcher
            .fetch_with_task(task)
            .await
            .map_err(|e| e.to_string())
    }

    async fn parse_task(&mut self, task: &mut Task) -> Result<(), AppError> {
        let parser = self.task_worker_maps.get_parser();
        parser.parse_with_task(task).await?;
        Ok(())
    }

    async fn insert_task(&mut self, task: &mut Task) -> Result<(), AppError> {
        let inserter = self.task_worker_maps.get_inserter();
        task.add_stage("inserting");
        if let Err(e) = inserter.insert(task.clone()).await {
            error!(
                worker_id = self.id,
                task_id = task.id,
                "Insert failed: {}",
                e
            );
            task.fail_stage(e.to_string());
            return Err(DomainError::new(
                DomainErrorKind::BatchProcessing,
                "insert submit fail",
                None,
                Some(Box::new(e)),
            )
            .into());
        }
        Ok(())
    }

    pub async fn update_history(&mut self, url: &str) {
        self.task_worker_maps
            .push_to_worker_with_capacity(self.id, url.to_string(), self.max_history_size)
            .await;
    }

    pub async fn calculate_similarity(&self, url: &str) -> f64 {
        if let Some(handled_tasks) = self.task_worker_maps.read_worker(&self.id).await {
            if handled_tasks.is_empty() {
                return 0.0;
            }

            handled_tasks
                .iter()
                .filter(|&handled| handled == url)
                .count() as f64
                + 1.0 / handled_tasks.len() as f64
        } else {
            0.0
        }
    }

    pub fn get_metrics(&self) -> &WorkerMetrics {
        &self.metrics
    }
}
