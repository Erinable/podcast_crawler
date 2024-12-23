use std::sync::Arc;

use futures::future::join_all;
use serde_json::json;
use tokio::sync::broadcast;

use super::{task::Task, task_management_system::TaskWorkerMaps, worker::Worker};

/// Internal Distributor structure
pub(crate) struct Distributor {
    task_id_counter: u64,
    task_tx: broadcast::Sender<Task>,
    task_worker_maps: Arc<TaskWorkerMaps>,
}

impl Distributor {
    pub(crate) fn new(
        task_tx: broadcast::Sender<Task>,
        task_worker_maps: Arc<TaskWorkerMaps>,
    ) -> Self {
        println!("🏭 Distributor: Creating new instance");
        Self {
            task_id_counter: 0,
            task_tx,
            task_worker_maps,
        }
    }

    async fn find_best_worker(&self, workers: &[Worker], url: &str) -> usize {
        println!("🔍 Distributor: Finding best worker for URL '{}'", url);
        let similarity_scores: Vec<f64> = join_all(
            workers
                .iter()
                .map(|worker| worker.calculate_similarity(url)),
        )
        .await;

        let best_worker_index = similarity_scores
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .map(|(index, _)| index)
            .unwrap_or(0);

        println!(
            "🔍 Distributor: Best worker index for URL '{}': {}",
            url, best_worker_index
        );
        best_worker_index
    }

    pub async fn create_task(&mut self, url: &str, workers: &mut [Worker]) {
        println!("📦 Distributor: Creating task for URL '{}'", url);

        // Create a new task
        self.task_id_counter += 1;
        let mut new_task = Task::new(self.task_id_counter, url.to_string(), 3);
        new_task.add_stage("distribution");
        let best_worker_id = self.find_best_worker(workers, url).await;
        new_task.target_thread_id = best_worker_id;
        new_task.complete_stage(json!({}));
        new_task.add_stage("processing");
        self.task_worker_maps
            .insert_task(new_task.get_id(), new_task.clone())
            .await;

        println!(
            "🎯 Distributor: Assigned task {} to worker {}",
            new_task.get_id(),
            best_worker_id
        );

        if let Err(e) = self.task_tx.send(new_task.clone()) {
            new_task.fail_stage(e.to_string());
            self.task_worker_maps
                .insert_task(new_task.get_id(), new_task)
                .await;
            eprintln!("❌ Distributor: Failed to send task: {}", e);
        }
    }
}
