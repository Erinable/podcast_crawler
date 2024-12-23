use std::sync::Arc;

use tokio::sync::broadcast;
use tokio_util::{sync::CancellationToken, task::TaskTracker};

use super::{
    task::Task,
    task_management_system::{ShutdownCoordinator, TaskWorkerMaps},
    timer_queue::TimerQueue,
    worker::Worker,
};
/// Internal ThreadManager structure
pub(crate) struct ThreadManager {
    pub task_tx: broadcast::Sender<Task>,
    pub workers: Vec<Worker>,
    pub task_tracker: Arc<TaskTracker>,
    pub cancellation_token: CancellationToken,
    pub shutdown_coordinator: Arc<ShutdownCoordinator>,
    pub timer_queue: Arc<TimerQueue>,
}

impl ThreadManager {
    pub async fn new(
        task_tx: broadcast::Sender<Task>,
        worker_count: usize,
        max_history_size: usize,
        task_tracker: Arc<TaskTracker>,
        cancellation_token: CancellationToken,
        shutdown_coordinator: Arc<ShutdownCoordinator>,
        task_worker_maps: Arc<TaskWorkerMaps>,
    ) -> Self {
        let mut workers = Vec::new();
        for i in 0..worker_count {
            task_worker_maps.insert_worker(i).await;
        }
        for i in 0..worker_count {
            workers.push(Worker::new(i, max_history_size, task_worker_maps.clone()));
        }
        let timer_queue = Arc::new(TimerQueue::new(
            task_tx.clone(),
            cancellation_token.clone(),
            shutdown_coordinator.clone(),
        ));
        Self {
            task_tx,
            workers,
            task_tracker,
            cancellation_token,
            shutdown_coordinator,
            timer_queue,
        }
    }

    pub async fn start(&mut self) {
        let timer_queue = self.timer_queue.clone();
        println!("⏰ Starting timer queue worker");
        self.task_tracker
            .spawn(async move { timer_queue.start_worker().await });
        println!(
            "🚀 ThreadManager: Starting workers. Total workers: {}",
            self.workers.len()
        );

        for worker in self.workers.iter_mut() {
            let worker_cancellation_token = self.cancellation_token.clone();
            let timer_queue = self.timer_queue.clone();
            let worker_task_rx = self.task_tx.subscribe();
            let shutdown_coordinator = self.shutdown_coordinator.clone();

            // Safely clone the worker
            let mut worker_clone = worker.clone();

            self.task_tracker.spawn(async move {
                worker_clone
                    .new_start(
                        worker_task_rx,
                        worker_cancellation_token,
                        timer_queue,
                        shutdown_coordinator,
                    )
                    .await
            });
        }
        println!("🎉 ThreadManager start completed");
    }
}
