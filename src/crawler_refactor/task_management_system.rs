use super::distributor::Distributor;
use super::inserter_refactored::BatchInserter;
use super::pipeline::{Fetcher, Parser};
use super::rss::RssFeedParser;
use super::rss_fetcher::RssFetcher;
use super::thread_manager::ThreadManager;
use crate::crawler_refactor::task::Task;
use crate::infrastructure::persistence::models::{NewEpisode, NewPodcast};
use crate::infrastructure::AppState;
use serde::Deserialize;
use std::collections::{HashMap, VecDeque};
use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{broadcast, RwLock};
use tokio_util::sync::CancellationToken;
use tokio_util::task::TaskTracker;

pub struct ShutdownCoordinator {
    pub worker_count: AtomicUsize,
    pub timer_queue_notify: CancellationToken,
    pub shutdown_complete: tokio::sync::Notify,
}

impl ShutdownCoordinator {
    pub async fn wait_for_timer_queue(&self) -> bool {
        let timeout = Duration::from_secs(100);
        tokio::select! {
            _ = self.timer_queue_notify.cancelled() => {
                // 收到通知
                true
            }
            _ = tokio::time::sleep(timeout) => {
                // 超时
                false
            }
        }
    }

    pub fn worker_completed(&self) {
        let remaining = self.worker_count.fetch_sub(1, Ordering::SeqCst) - 1;
        if remaining == 0 {
            self.shutdown_complete.notify_one();
        }
    }
}

#[derive(Deserialize, Debug)]
struct ResultData {
    podcast: NewPodcast,
    episodes: Vec<NewEpisode>,
}

#[derive(Clone, Debug)]
pub struct TaskWorkerMaps {
    worker_metadata: Arc<RwLock<HashMap<usize, RwLock<VecDeque<String>>>>>,
    task_metadata: Arc<RwLock<HashMap<u64, RwLock<Task>>>>,
    fetcher: Arc<dyn Fetcher + Send + Sync>,
    parser: Arc<dyn Parser<(NewPodcast, Vec<NewEpisode>)> + Send + Sync>,
    batch_inserter: Arc<BatchInserter>,
}

impl Default for TaskWorkerMaps {
    fn default() -> Self {
        panic!("TaskWorkerMaps::default() should not be used directly. Use TaskWorkerMaps::new(state) instead.");
    }
}

fn create_process_batch_fn(
    state: Arc<AppState>,
) -> impl Fn(Vec<Task>) -> Pin<Box<dyn Future<Output = Result<(), String>> + Send>> + Clone {
    move |batch: Vec<Task>| {
        let state = state.clone();
        Box::pin(async move {
            let podcast_repo = &state.repositories.podcast;

            for mut task in batch {
                if let Some(result_data) = task.get_stage_result_data_by_name("parsing") {
                    // 解码 JSON 数据
                    if let Ok(result) = serde_json::from_value::<ResultData>(result_data.clone()) {
                        // 插入数据库
                        match podcast_repo
                            .insert_with_episodes(&result.podcast, &result.episodes)
                            .await
                        {
                            Ok(_) => {
                                if task.get_task_status() == super::task::StageStatus::InProgress {
                                    task.complete_stage(serde_json::json!({"status": "success"}));
                                }
                            }
                            Err(e) => {
                                if task.get_task_status() == super::task::StageStatus::InProgress {
                                    task.fail_stage(format!("Failed to insert podcast: {}", e));
                                }
                            }
                        }
                    } else if task.get_task_status() == super::task::StageStatus::InProgress {
                        task.fail_stage("Failed to decode podcast data".to_string());
                    }
                } else {
                    task.fail_stage("No result data available".to_string());
                }
            }

            Ok(())
        })
    }
}

impl TaskWorkerMaps {
    pub fn new(state: Arc<AppState>) -> Self {
        let fetcher = Arc::new(RssFetcher::new());
        let parser = Arc::new(RssFeedParser::new());

        // Initialize batch inserter
        let batch_inserter = Arc::new(BatchInserter::new(
            3,  // batch size
            10, // max concurrent inserts
            create_process_batch_fn(state.clone()),
            Duration::from_secs(5), // batch timeout
        ));

        TaskWorkerMaps {
            worker_metadata: Arc::new(RwLock::new(HashMap::new())),
            task_metadata: Arc::new(RwLock::new(HashMap::new())),
            fetcher,
            parser,
            batch_inserter,
        }
    }

    // Insert an empty Vec into map_vec
    pub async fn insert_worker(&self, key: usize) {
        let mut map = self.worker_metadata.write().await;
        map.insert(key, RwLock::new(Vec::new().into()));
    }

    // Insert a MyStruct into map_struct
    pub async fn insert_task(&self, key: u64, value: Task) {
        let mut map = self.task_metadata.write().await;
        map.insert(key, RwLock::new(value));
    }

    // Push a value into the Vec associated with a key in map_vec
    pub async fn push_to_worker(&self, key: usize, value: String) {
        if let Some(lock) = self.worker_metadata.read().await.get(&key) {
            let mut vec = lock.write().await;
            vec.push_back(value);
        }
    }

    // Push a value into the Vec associated with a key in map_vec
    pub async fn push_to_worker_with_capacity(&self, key: usize, value: String, capacity: usize) {
        if let Some(lock) = self.worker_metadata.read().await.get(&key) {
            let mut vec = lock.write().await;
            while vec.len() > capacity {
                vec.pop_front();
            }
            vec.push_back(value);
        }
    }

    // Update the MyStruct value associated with a key in map_struct
    pub async fn update_task(&self, key: u64, value: Task) {
        if let Some(lock) = self.task_metadata.read().await.get(&key) {
            let mut struct_value = lock.write().await;
            *struct_value = value;
        }
    }

    // Read the Vec for a key from map_vec
    pub async fn read_worker(&self, key: &usize) -> Option<Vec<String>> {
        if let Some(lock) = self.worker_metadata.read().await.get(key) {
            let vec = lock.read().await;
            Some(vec.clone().into())
        } else {
            None
        }
    }

    // Read the MyStruct for a key from map_struct
    pub async fn read_task(&self, key: &u64) -> Option<Task> {
        if let Some(lock) = self.task_metadata.read().await.get(key) {
            let struct_value = lock.read().await;
            Some((*struct_value).clone())
        } else {
            None
        }
    }

    pub async fn read_all_tasks(&self) -> Vec<Task> {
        let map = self.task_metadata.read().await;
        futures::future::join_all(
            map.values()
                .map(|lock| async move { lock.read().await.clone() }),
        )
        .await
    }

    pub fn get_fetcher(&self) -> Arc<dyn Fetcher + Send + Sync> {
        self.fetcher.clone()
    }

    pub fn get_parser(&self) -> Arc<dyn Parser<(NewPodcast, Vec<NewEpisode>)> + Send + Sync> {
        self.parser.clone()
    }

    pub fn get_inserter(&self) -> Arc<BatchInserter> {
        self.batch_inserter.clone()
    }
}

/// Public-facing TaskManagementSystem structure
pub struct TaskManagementSystem {
    distributor: Distributor,
    thread_manager: ThreadManager,
    task_tracker: Arc<TaskTracker>,
    cancellation_token: CancellationToken,
    task_worker_maps: Arc<TaskWorkerMaps>,
}

impl TaskManagementSystem {
    pub async fn new(state: Arc<AppState>, worker_count: usize, max_history_size: usize) -> Self {
        tracing::info!(
            "🚦 TaskManagementSystem: Initializing with {} workers",
            worker_count
        );

        let task_tracker = Arc::new(TaskTracker::new());
        let cancellation_token = CancellationToken::new();
        let (task_tx, _task_rx) = broadcast::channel::<Task>(5000);
        let task_worker_maps = Arc::new(TaskWorkerMaps::new(state.clone()));
        let shutdown_coordinator = Arc::new(ShutdownCoordinator {
            worker_count: AtomicUsize::new(worker_count),
            timer_queue_notify: CancellationToken::new(),
            shutdown_complete: tokio::sync::Notify::new(),
        });
        let distributor = Distributor::new(task_tx.clone(), task_worker_maps.clone());

        let thread_manager = ThreadManager::new(
            task_tx,
            worker_count,
            max_history_size,
            task_tracker.clone(),
            cancellation_token.clone(),
            shutdown_coordinator.clone(),
            task_worker_maps.clone(),
        )
        .await;

        tracing::info!("🎉 TaskManagementSystem: Initialization complete");

        Self {
            distributor,
            thread_manager,
            task_tracker,
            cancellation_token,
            task_worker_maps,
        }
    }

    // Async initialization method
    pub async fn start(&mut self) {
        tracing::info!("🔥 TaskManagementSystem: Starting system");
        self.thread_manager.start().await;
        tracing::info!("✅ TaskManagementSystem: System started successfully");
    }

    /// Add a new task
    pub async fn add_task(&mut self, url: &str) -> Result<(), String> {
        tracing::info!("➕ TaskManagementSystem: Adding task for URL '{}'", url);

        // Create a mutable reference to workers
        let mut workers = self.thread_manager.workers.clone();

        // Use the distributor to create and distribute the task
        match self.distributor.create_task(url, &mut workers).await {
            Ok(_) => {
                tracing::info!(
                    "🚀 TaskManagementSystem: Task for '{}' added successfully",
                    url
                );
                Ok(())
            }
            Err(e) => {
                tracing::error!(
                    "❌ TaskManagementSystem: Failed to add task for '{}': {}",
                    url,
                    e
                );
                Err(e)
            }
        }
    }

    // Get real-time task metadata
    pub async fn get_task_info(&self) -> Vec<Task> {
        tracing::info!("📋 TaskManagementSystem: Retrieving task information");
        let task_info = self.task_worker_maps.read_all_tasks().await;
        tracing::info!(
            "📊 TaskManagementSystem: Retrieved {} tasks",
            task_info.len()
        );
        task_info
    }

    /// Wait for all tasks to complete and return the list of tasks
    pub async fn wait_for_all_tasks_completed(&self) -> Vec<Task> {
        self.wait_for_all_tasks_completed_with_timeout(Duration::from_secs(300))
            .await
    }

    /// Wait for all tasks to complete with a custom timeout
    pub async fn wait_for_all_tasks_completed_with_timeout(&self, timeout: Duration) -> Vec<Task> {
        tracing::info!(
            "⏳ TaskManagementSystem: Waiting for all tasks to complete (timeout: {:?})",
            timeout
        );
        tracing::info!("🕵️ Diagnostic info:");

        tracing::info!(
            "   - Cancellation token cancelled: {}",
            self.cancellation_token.is_cancelled()
        );

        let start_time = std::time::Instant::now();

        // Implement a timeout mechanism
        let timeout_result = tokio::time::timeout(timeout, self.task_tracker.wait()).await;

        match timeout_result {
            Ok(_) => {
                let duration = start_time.elapsed();
                tracing::info!("✅ TaskManagementSystem: tasks completed in {:?}", duration);

                // Retrieve and log task metadata
                let completed_tasks = self.task_worker_maps.read_all_tasks().await;

                tracing::info!("📊 Task Completion Summary:");
                tracing::info!("   - Total tasks: {}", completed_tasks.len());
                tracing::info!(
                    "   - Successful tasks: {}",
                    completed_tasks.iter().filter(|t| t.is_completed()).count()
                );
                tracing::info!(
                    "   - Failed tasks: {}",
                    completed_tasks.iter().filter(|t| t.is_failed()).count()
                );

                completed_tasks
            }
            Err(_) => {
                tracing::error!("❌ TaskManagementSystem: Timeout waiting for tasks to complete");
                tracing::info!("🚨 Diagnostic details at timeout:");
                tracing::info!("   - Elapsed time: {:?}", start_time.elapsed());

                // Attempt to force shutdown
                self.cancellation_token.cancel();

                // Return any tasks that have been processed
                self.task_worker_maps.read_all_tasks().await
            }
        }
    }

    /// Gracefully shut down the system
    pub async fn shutdown(&self) {
        self.shutdown_with_timeout(Duration::from_secs(20)).await
    }

    /// Gracefully shut down the system with a custom timeout
    pub async fn shutdown_with_timeout(&self, timeout: Duration) {
        tracing::info!(
            "🛑 TaskManagementSystem: Initiating shutdown (timeout: {:?})",
            timeout
        );

        // Log detailed system state before shutdown
        tracing::info!("🔍 Pre-shutdown system state:");
        tracing::info!(
            "   - Cancellation token cancelled: {}",
            self.cancellation_token.is_cancelled()
        );

        // Cancel all ongoing tasks
        self.cancellation_token.cancel();

        // Wait for tasks with a timeout
        let shutdown_result = tokio::time::timeout(timeout, async {
            self.task_tracker.close();
            self.task_tracker.wait().await;
        })
        .await;

        match shutdown_result {
            Ok(_) => {
                tracing::info!("👋 TaskManagementSystem: Shutdown completed successfully");
            }
            Err(_) => {
                tracing::error!("❌ TaskManagementSystem: Shutdown timed out");
                tracing::info!("🚨 Post-timeout system state:");
                tracing::info!(
                    "   - Remaining tasks: {}",
                    self.task_worker_maps.read_all_tasks().await.len()
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::initialize;
    use std::time::Duration;
    use tokio::runtime::Runtime;

    #[tokio::test]
    async fn test_task_creation_and_distribution() {
        // Initialize system with 3 workers
        let state = initialize().await.unwrap();
        let mut system = TaskManagementSystem::new(Arc::new(state), 3, 10).await;
        system.start().await;

        // Add some tasks
        system.add_task("http://example1.com").await;
        system
            .add_task("https://justpodmedia.com/rss/middle-ground.xml")
            .await;
        // system.add_task("http://example3.com").await;

        // Allow some time for tasks to be processed
        tokio::time::sleep(Duration::from_millis(1000)).await;
        // system.add_task("http://example4.com").await;
        // system.add_task("http://example5.com").await;
        // tokio::time::sleep(Duration::from_millis(1000)).await;
        system.add_task("http://example6.com").await;
        system.add_task("http://example7.com").await;
        tokio::time::sleep(Duration::from_millis(1000)).await;
        system.add_task("").await;
        system.add_task("").await;
        // Check task metadata
        let _a = system.wait_for_all_tasks_completed().await;
        system.shutdown().await;
        let task_info = system.get_task_info().await;
        assert_eq!(task_info.len(), 6);
        println!("{:#?}", task_info);
        // Print detailed stages information for each task
        // println!("📋 Task Stages Information:");
        // for (i, task) in task_info.iter().enumerate() {
        //     println!("  Task {} stages:", i + 1);
        //     for (j, stage) in task.stages.iter().enumerate() {
        //         println!("    Stage {}: {:?}", j + 1, stage);
        //     }
        // }
    }

    #[test]
    fn test_worker_load_balancing() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let state = initialize().await.unwrap();
            let mut system = TaskManagementSystem::new(Arc::new(state), 2, 5).await;
            system.start().await;

            // Add multiple tasks with the same URL to test worker selection
            let test_url = "http://example.com";
            for _ in 0..5 {
                tokio::time::sleep(Duration::from_millis(100)).await;
                system.add_task(test_url).await;
            }

            tokio::time::sleep(Duration::from_millis(100)).await;
            let task_info = system.get_task_info().await;

            // Count tasks per worker
            let mut worker_task_counts = [0; 2];
            for task in task_info {
                worker_task_counts[task.target_thread_id] += 1;
            }

            // Verify tasks are somewhat evenly distributed
            assert!(worker_task_counts.iter().all(|&count| count > 0));
            system.shutdown().await;
            let task_info = system.get_task_info().await;
            println!("{:#?}", task_info);
        });
    }

    #[test]
    fn test_task_retry_mechanism() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let state = initialize().await.unwrap();
            println!("🔍 Starting TaskManagementSystem test");
            let mut system = TaskManagementSystem::new(Arc::new(state), 1, 5).await;
            system.start().await;

            // Add a task with empty payload (which should fail)
            println!("🚀 Adding task with empty payload");
            system.add_task("").await;

            // Wait for initial attempt and retries
            println!("⏳ Waiting for retry attempts");
            tokio::time::sleep(Duration::from_secs(3)).await;

            let task_info = system.get_task_info().await;
            let task = task_info.first().unwrap();

            // Check if retries occurred
            println!("📊 Task retry count: {}", task.retries);
            assert!(
                task.retries > 0,
                "Task should have been retried at least once"
            );

            println!("✅ Retry mechanism test completed");
            system.shutdown().await;
        });
    }

    #[test]
    fn test_system_shutdown() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let state = initialize().await.unwrap();
            let mut system = TaskManagementSystem::new(Arc::new(state), 2, 5).await;
            system.start().await;

            // Add some tasks
            for i in 0..5 {
                system.add_task(&format!("http://example{}.com", i)).await;
            }

            // Immediate shutdown
            system.shutdown().await;

            // Verify system state after shutdown
            let tasks = system.wait_for_all_tasks_completed().await;
            assert!(!tasks.is_empty());

            // Try to add a task after shutdown (should not panic)
            system.add_task("http://example.com").await;
        });
    }

    #[test]
    fn test_worker_history() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let state = initialize().await.unwrap();
            let mut system = TaskManagementSystem::new(Arc::new(state), 1, 3).await;
            system.start().await;

            // Add more tasks than the history size
            let test_url = "http://example.com";
            for _ in 0..5 {
                system.add_task(test_url).await;
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
            system.shutdown().await;
        });
    }

    #[test]
    fn test_task_metadata_tracking() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let state = initialize().await.unwrap();
            let mut system = TaskManagementSystem::new(Arc::new(state), 1, 5).await;
            system.start().await;

            // Add a task and track its progress
            system.add_task("http://example.com").await;

            // Initial state check
            let initial_info = system.get_task_info().await;
            let task = initial_info.first().unwrap();
            assert!(task
                .stages
                .iter()
                .any(|stage| stage.name.contains("distribution")));

            // Wait for processing
            tokio::time::sleep(Duration::from_millis(100)).await;

            // Final state check
            let final_info = system.get_task_info().await;
            assert!(
                !final_info.is_empty(),
                "Final task info should not be empty"
            );
            assert!(final_info.len() == 1, "Should have exactly one task");
            let final_task = final_info.first().unwrap();
            assert!(
                final_task.stages.len() > 1,
                "Task should have progressed through multiple stages"
            );

            system.shutdown().await;
        });
    }
}
