use crate::crawler_refactor::task::Task;
use crate::crawler_refactor::task_management_system::ShutdownCoordinator;
use std::collections::BinaryHeap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::time::Instant;
use tokio::sync::broadcast;
use tokio_util::sync::CancellationToken;

#[derive(Clone)]
pub struct TimerQueue {
    timers: Arc<Mutex<BinaryHeap<Task>>>,
    worker_task_tx: broadcast::Sender<Task>,
    cancellation_token: CancellationToken,
    shutdown_coordinator: Arc<ShutdownCoordinator>,
}
impl TimerQueue {
    pub fn new(
        worker_task_tx: broadcast::Sender<Task>,
        cancellation_token: CancellationToken,
        shutdown_coordinator: Arc<ShutdownCoordinator>,
    ) -> Self {
        Self {
            timers: Arc::new(Mutex::new(BinaryHeap::new())),
            worker_task_tx,
            cancellation_token,
            shutdown_coordinator,
        }
    }
    pub fn schedule_retry(&self, mut task: Task) {
        // Ensure backoff timer is set
        if task.backoff_timer.is_none() {
            task.backoff_timer = Some(Instant::now() + Duration::from_secs(1));
        }
        let mut timers = self.timers.lock().unwrap();
        timers.push(task);
        crate::metrics::TASK_RETRIES.inc();
        tracing::debug!(
            "⏰ TimerQueue: Task retries count: {}",
            crate::metrics::TASK_RETRIES.get()
        );
    }

    pub async fn start_worker(&self) {
        // let mut last_log_time = Instant::now();
        loop {
            tokio::select! {
                _ = self.cancellation_token.cancelled() => {
                    tracing::info!("🛑 Timer queue received cancellation signal");
                    // Drain and process all remaining tasks
                    let remaining_tasks = {
                        let mut timers = self.timers.lock().unwrap();
                        let mut tasks = Vec::new();
                        while let Some(mut task) = timers.pop() {
                            task.shutdown = true;
                            tasks.push(task);
                        }
                        tasks
                    };

                    tracing::info!("📤 Processing {} remaining tasks before shutdown", remaining_tasks.len());
                    for task in remaining_tasks {
                        if let Err(e) = self.worker_task_tx.send(task) {
                            tracing::error!("❌ Failed to send task during shutdown: {}", e);
                        }
                    }
                    // Signal that timer queue processing is complete
                    self.shutdown_coordinator.timer_queue_notify.cancel();
                    tracing::info!("✅ Timer queue shutdown completed");
                    break;
                }
               () = async {
                let next_task = {
                    let mut timers = self.timers.lock().unwrap();
                    // // Log queue size every 5 seconds
                    // let now = Instant::now();
                    // if now.duration_since(last_log_time) >= Duration::from_secs(5) {
                    //     tracing::debug!("⏰ TimerQueue: Current timer queue size: {}", timers.len());
                    //     last_log_time = now;
                    // }

                    // Get next task if it's ready
                    if let Some(timer) = timers.peek() {
                        if let Some(backoff_time) = timer.backoff_timer {
                            if backoff_time <= Instant::now() {
                                tracing::debug!("⏰ TimerQueue: Task ready for processing");
                                timers.pop()
                            } else {
                                None
                            }
                        } else {
                            tracing::warn!("⏰ TimerQueue: Task has no backoff timer set");
                            None
                        }
                    } else {
                        None
                    }
                };

                match next_task {
                    Some(task) => {
                        if let Err(e) = self.worker_task_tx.send(task) {
                            tracing::error!("❌ TimerQueue: Failed to send retry task: {}", e);
                        } else {
                            tracing::debug!("✅ TimerQueue: Retry task sent successfully");
                        }
                    }
                    None => {
                        // Sleep before checking again
                        tokio::time::sleep(Duration::from_millis(100)).await;
                    }
                }
               } => {}
            }
        }
        tracing::info!("🏁 TimerQueue: Worker stopped completely.");
    }
}
