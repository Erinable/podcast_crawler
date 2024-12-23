use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use serde_json::json;
use tokio::{sync::broadcast, time::timeout};
use tokio_util::sync::CancellationToken;

use super::{
    task::Task,
    task_management_system::{ShutdownCoordinator, TaskWorkerMaps},
    timer_queue::TimerQueue,
};

/// Internal Worker structure
#[derive(Debug, Clone)]
pub struct Worker {
    id: usize,
    max_history_size: usize,
    in_progress_count: usize,
    task_worker_maps: Arc<TaskWorkerMaps>,
}

impl Worker {
    pub fn new(id: usize, max_history_size: usize, task_worker_maps: Arc<TaskWorkerMaps>) -> Self {
        Self {
            id,
            in_progress_count: 0,
            max_history_size,
            task_worker_maps,
        }
    }

    pub async fn new_start(
        &mut self,
        mut worker_task_rx: broadcast::Receiver<Task>,
        worker_cancellation_token: CancellationToken,
        timer_queue: Arc<TimerQueue>,
        shutdown_coordinator: Arc<ShutdownCoordinator>,
    ) {
        println!("🔧 Worker {} starting task processing loop", self.id);
        // Keep track of tasks that were in progress when shutdown started
        let mut in_progress_tasks = Vec::new();
        let mut draining = false; // Flag to indicate draining state
        loop {
            tokio::select! {
                Ok(mut task) = worker_task_rx.recv() => {
                    if !draining && task.target_thread_id != self.id {
                        println!("🔀 Worker {} skipping task (not target thread)", self.id);
                        continue;
                    }
                    if !draining {
                        println!("📋 Worker {} processing task: {}", self.id, task.payload);
                        self.in_progress_count += 1;
                        // Track task as in-progress
                        in_progress_tasks.push(task.get_id());
                        let process_result = self.process_task(&mut task).await;
                        self.in_progress_count -= 1;
                        // Remove task from in-progress tracking
                        if let Some(pos) = in_progress_tasks.iter().position(|x| *x == task.get_id()) {
                            in_progress_tasks.remove(pos);
                        }
                        match process_result {
                            Ok(_) => {
                                task.complete_stage(json!({}));
                                self.task_worker_maps.update_task(task.get_id(),task.clone()).await;
                                self.update_history(&task.payload).await;
                            }
                            Err(e) => {
                                self.handle_task_error(&mut task, e, &timer_queue).await;
                            }
                        }
                    }

                }
                _ = worker_cancellation_token.cancelled() => {
                    draining = true;
                    println!("Worker {} received cancellation signal, entering drain mode:{}", self.id,draining);


                    // Wait for timer queue to finish processing
                    let wait_result = shutdown_coordinator.wait_for_timer_queue().await;
                    match wait_result {
                        true => {},
                        false => {println!("Timer queue failed to complete time out"); break},
                    }
                    // Mark all in-progress tasks as failed due to shutdown
                    for task_id in in_progress_tasks.iter() {
                        if let Some(mut task) = self.task_worker_maps.read_task(task_id).await {
                            task.error_message = Some("shutdown signal".to_string());
                            task.fail_stage("shutdown signal".to_string());
                            self.task_worker_maps.update_task(task.get_id(),task).await;
                        }
                    }

                    loop {
                        match timeout(Duration::from_secs(5), worker_task_rx.recv()).await {
                            Ok(Ok(mut task)) => {
                                if task.target_thread_id != self.id {
                                    println!("🔀 Worker {} skipping task (not target thread)", self.id);
                                    continue;
                                }

                                if task.shutdown {
                                    println!("receive flush data from timer queue");
                                    task.error_message = Some("shutdown signal".to_string());
                                    task.fail_stage("shutdown signal".to_string());
                                    self.task_worker_maps.update_task(task.get_id(),task.clone()).await;
                                    continue;
                                }
                            }
                            Ok(Err(_)) | Err(_) => {
                                // Timeout or channel closure
                                println!("Worker {} no more tasks to drain, exiting", self.id);
                                break;
                            }
                        }
                    }

                    println!("Worker {} drain phase complete, exiting", self.id);
                    shutdown_coordinator.worker_completed();
                    break;
                }
            }
        }
    }

    // Helper method to handle task errors
    async fn handle_task_error(&self, task: &mut Task, e: String, timer_queue: &Arc<TimerQueue>) {
        println!("❌ Worker {} task failed: {}", self.id, e);
        if task.retries < task.max_retries {
            task.retries += 1;
            println!(
                "🔄 Worker {} retrying task. Retry count: {}",
                self.id, task.retries
            );
            task.backoff_timer = Some(Instant::now() + Duration::from_secs(1));
            timer_queue.schedule_retry(task.clone());
        } else {
            println!("❗ Worker {} max retries reached", self.id);
            task.error_message = Some(e.clone());
            task.fail_stage(e);
        }
        self.task_worker_maps
            .update_task(task.get_id(), task.clone())
            .await;
    }

    async fn process_task(&mut self, task: &mut Task) -> Result<(), String> {
        // Implement your task processing logic here
        // This is a placeholder implementation
        match task.payload.len() {
            0 => Err("Empty task payload".to_string()),
            _ => Ok(()),
        }
    }

    pub async fn update_history(&mut self, url: &str) {
        self.task_worker_maps
            .push_to_worker_with_capacity(self.id, url.to_string(), self.max_history_size)
            .await;
    }

    pub async fn calculate_similarity(&self, url: &str) -> f64 {
        if let Some(handled_tasks) = self.task_worker_maps.read_worker(&self.id).await {
            println!(
                "🔍 Worker {} calculating similarity for URL '{}', handle list:{:#?}",
                self.id, url, handled_tasks
            );

            if handled_tasks.is_empty() {
                return 0.0;
            }

            // Calculate similarity while holding the lock
            let similarity = handled_tasks
                .iter()
                .filter(|&handled| handled == url)
                .count() as f64
                + 1.0 / handled_tasks.len() as f64;
            similarity
        } else {
            0.0
        }
    }
}
