use std::sync::Arc;

use futures::future::join_all;
use rand::Rng;
use serde_json::json;
use tokio::sync::broadcast;

use super::{task::Task, task_management_system::TaskWorkerMaps, worker::Worker};

/// Internal Distributor structure
pub(crate) struct Distributor {
    task_id_counter: u64,
    task_tx: broadcast::Sender<Task>,
    task_worker_maps: Arc<TaskWorkerMaps>,
    current_index: usize,
}

impl Distributor {
    pub(crate) fn new(
        task_tx: broadcast::Sender<Task>,
        task_worker_maps: Arc<TaskWorkerMaps>,
    ) -> Self {
        tracing::info!("🏭 Distributor: Creating new instance");
        Self {
            task_id_counter: 0,
            task_tx,
            task_worker_maps,
            current_index: 0,
        }
    }

    // 选择工作线程 ID，使用轮询方式
    fn select_worker(&mut self, workers: &[Worker]) -> usize {
        // 获取当前工作线程的 ID
        let worker_id = workers[self.current_index].id;

        // 更新索引到下一个工作线程，循环到第一个
        self.current_index = (self.current_index + 1) % workers.len();

        worker_id
    }

    async fn find_best_worker(&self, workers: &[Worker], url: &str) -> usize {
        tracing::info!("🔍 Distributor: Finding best worker for URL '{}'", url);
        // 设置探索概率 ε
        let epsilon = 0.4; // 10% 的概率进行随机选择
        let mut rng = rand::thread_rng();
        // Get both similarity scores and queue lengths
        let worker_metrics: Vec<(f64, usize)> = join_all(workers.iter().map(|worker| async move {
            let similarity = worker.calculate_similarity(url).await;
            let queue_length = self
                .task_worker_maps
                .read_worker(&worker.id)
                .await
                .map_or(0, |tasks| tasks.len());
            (similarity, queue_length)
        }))
        .await;
        // 生成随机数以决定是否进行随机选择
        let random_number: f64 = rng.gen();
        // 生成 0 到 1 之间的随机数
        // Find worker with best combination of similarity and queue length
        let best_worker_index = if random_number < epsilon {
            // 随机选择一个工作线程
            rng.gen_range(0..workers.len())
        } else {
            // 找到具有最佳组合的工作线程
            worker_metrics
                .iter()
                .enumerate()
                .min_by(|(_, (a_sim, a_len)), (_, (b_sim, b_len))| {
                    // 优先考虑较短的队列长度
                    if a_len != b_len {
                        a_len.cmp(b_len)
                    } else {
                        // 如果队列长度相同，优先考虑更高的相似度
                        a_sim.partial_cmp(b_sim).unwrap()
                    }
                })
                .map(|(index, _)| index)
                .unwrap_or(0) // 默认返回第一个工作线程
        };

        tracing::info!(
            "🔍 Distributor: Best worker index for URL '{}': {} (similarity: {}, queue length: {})",
            url,
            best_worker_index,
            worker_metrics[best_worker_index].0,
            worker_metrics[best_worker_index].1
        );
        best_worker_index
    }

    pub async fn create_task(&mut self, url: &str, workers: &mut [Worker]) -> Result<(), String> {
        tracing::info!("📦 Distributor: Creating task for URL '{}'", url);

        // Create a new task
        self.task_id_counter += 1;
        let mut new_task = Task::new(self.task_id_counter, url.to_string(), 0);
        new_task.add_stage("distribution");
        // let best_worker_id = self.find_best_worker(workers, url).await;
        let best_worker_id = self.select_worker(workers);

        // Only complete stage if still in progress
        if new_task.get_task_status() == super::task::StageStatus::InProgress {
            new_task.complete_stage(json!({}));
        }

        new_task.target_thread_id = best_worker_id;

        self.task_worker_maps
            .insert_task(new_task.get_id(), new_task.clone())
            .await;

        tracing::info!(
            "🎯 Distributor: Assigned task {} to worker {}",
            new_task.get_id(),
            best_worker_id
        );

        match self.task_tx.send(new_task.clone()) {
            Ok(_) => Ok(()),
            Err(e) => {
                new_task.fail_stage(e.to_string());
                self.task_worker_maps
                    .insert_task(new_task.get_id(), new_task)
                    .await;
                tracing::error!("❌ Distributor: Failed to send task: {}", e);
                Err(e.to_string())
            }
        }
    }
}
