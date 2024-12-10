use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

use crate::crawler::TaskResult;

pub(crate) struct BatchStats {
    start_time: Instant,
    total_count: Arc<AtomicUsize>,
    success_count: Arc<AtomicUsize>,
    failure_count: Arc<AtomicUsize>,
    error_counts: Arc<Mutex<HashMap<String, usize>>>,
    total_duration: Arc<Mutex<Duration>>,
}

impl Default for BatchStats {
    fn default() -> Self {
        Self {
            start_time: Instant::now(),
            total_count: Arc::new(AtomicUsize::new(0)),
            success_count: Arc::new(AtomicUsize::new(0)),
            failure_count: Arc::new(AtomicUsize::new(0)),
            error_counts: Arc::new(Mutex::new(HashMap::new())),
            total_duration: Arc::new(Mutex::new(Duration::default())),
        }
    }
}

impl BatchStats {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn increment_total(&self) {
        self.total_count.fetch_add(1, Ordering::SeqCst);
    }

    pub fn increment_success(&self) {
        self.success_count.fetch_add(1, Ordering::SeqCst);
    }

    pub fn increment_failure(&self) {
        self.failure_count.fetch_add(1, Ordering::SeqCst);
    }

    pub async fn add_error(&self, error_type: &str) {
        let mut error_counts = self.error_counts.lock().await;
        *error_counts.entry(error_type.to_string()).or_insert(0) += 1;
    }

    pub async fn add_duration(&self, duration: Duration) {
        let mut total_duration = self.total_duration.lock().await;
        *total_duration += duration;
    }

    pub async fn record_result<T>(&self, result: &TaskResult<T>) {
        self.increment_total();

        if result.success {
            self.increment_success();
        } else {
            self.increment_failure();
            if let Some(error_msg) = &result.error_message {
                let error_type = categorize_error(error_msg);
                self.add_error(&error_type).await;
            }
        }

        self.add_duration(result.duration).await;
    }

    pub async fn get_summary(&self) -> StatsSummary {
        StatsSummary {
            total_tasks: self.total_count.load(Ordering::SeqCst),
            successful_tasks: self.success_count.load(Ordering::SeqCst),
            failed_tasks: self.failure_count.load(Ordering::SeqCst),
            total_duration: *self.total_duration.lock().await,
            error_counts: self.error_counts.lock().await.clone(),
            elapsed_time: self.start_time.elapsed(),
        }
    }
}

pub(crate) struct StatsSummary {
    pub total_tasks: usize,
    pub successful_tasks: usize,
    pub failed_tasks: usize,
    pub total_duration: Duration,
    pub error_counts: HashMap<String, usize>,
    pub elapsed_time: Duration,
}

impl StatsSummary {
    pub fn success_rate(&self) -> f64 {
        if self.total_tasks > 0 {
            (self.successful_tasks as f64 / self.total_tasks as f64) * 100.0
        } else {
            0.0
        }
    }

    pub fn average_task_duration(&self) -> Duration {
        if self.total_tasks > 0 {
            self.total_duration / self.total_tasks as u32
        } else {
            Duration::new(0, 0)
        }
    }

    pub fn format_report(&self) -> String {
        let mut report = String::new();
        report.push_str("\n=== Batch Processing Statistics ===\n");
        report.push_str(&format!("Total tasks: {}\n", self.total_tasks));
        report.push_str(&format!(
            "Successful tasks: {} ({:.1}%)\n",
            self.successful_tasks,
            self.success_rate()
        ));
        report.push_str(&format!("Failed tasks: {}\n", self.failed_tasks));
        report.push_str(&format!("Total time elapsed: {:?}\n", self.elapsed_time));
        report.push_str(&format!(
            "Average task duration: {:?}\n",
            self.average_task_duration()
        ));

        if !self.error_counts.is_empty() {
            report.push_str("\nError Distribution:\n");
            for (error_type, count) in &self.error_counts {
                let percentage = (*count as f64 / self.failed_tasks as f64) * 100.0;
                report.push_str(&format!(
                    "  {}: {} ({:.1}%)\n",
                    error_type, count, percentage
                ));
            }
        }

        report.push_str("==============================\n");
        report
    }
}

fn categorize_error(error_msg: &str) -> String {
    if error_msg.contains("timeout") {
        "Timeout".to_string()
    } else if error_msg.contains("connection refused") {
        "Connection Refused".to_string()
    } else if error_msg.contains("DNS") {
        "DNS Error".to_string()
    } else if error_msg.contains("Parse error") {
        "Parse Error".to_string()
    } else {
        "Other Error".to_string()
    }
}
