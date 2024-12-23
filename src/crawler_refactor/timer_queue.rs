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
    }

    pub async fn start_worker(&self) {
        loop {
            tokio::select! {
                _ = self.cancellation_token.cancelled() => {

                    println!("🛑 Timer queue received cancellation signal");
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

                    println!("📤 Processing {} remaining tasks before shutdown", remaining_tasks.len());
                    for task in remaining_tasks {
                        if let Err(e) = self.worker_task_tx.send(task) {
                            eprintln!("❌ Failed to send task during shutdown: {}", e);
                        }
                    }
                    // Signal that timer queue processing is complete
                    self.shutdown_coordinator.timer_queue_notify.cancel();
                    println!("✅ Timer queue shutdown completed");
                    break;
                }
               () = async {
                let next_task = {
                    let mut timers = self.timers.lock().unwrap();
                    println!("⏰ TimerQueue: Current timer queue size: {}", timers.len());

                    // Get next task if it's ready
                    if let Some(timer) = timers.peek() {
                        if let Some(backoff_time) = timer.backoff_timer {
                            println!("⏰ TimerQueue: Next task backoff time: {:?}", backoff_time);
                            println!("⏰ TimerQueue: Current time: {:?}", Instant::now());

                            if backoff_time <= Instant::now() {
                                println!("⏰ TimerQueue: Task ready for processing");
                                timers.pop()
                            } else {
                                println!("⏰ TimerQueue: Task not yet ready. Waiting...");
                                None
                            }
                        } else {
                            println!("⏰ TimerQueue: Task has no backoff timer set");
                            None
                        }
                    } else {
                        println!("⏰ TimerQueue: No tasks in queue");
                        None
                    }
                };

                match next_task {
                    Some(task) => {
                        println!("⏰ TimerQueue: Attempting to send retry task");
                        if let Err(e) = self.worker_task_tx.send(task) {
                            eprintln!("❌ TimerQueue: Failed to send retry task: {}", e);
                        } else {
                            println!("✅ TimerQueue: Retry task sent successfully");
                        }
                    }
                    None => {
                        // Sleep before checking again
                        println!("⏰ TimerQueue: No tasks ready. Sleeping...");
                        tokio::time::sleep(Duration::from_millis(100)).await;
                    }
                }
               } => {}
            }
        }
        println!("🏁 TimerQueue: Worker stopped completely.");
    }
}
