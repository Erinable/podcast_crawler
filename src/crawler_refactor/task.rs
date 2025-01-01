use serde_json::Value;
use std::cmp::Ordering;
use std::fmt::Debug;
use std::time::Instant;
use tracing::{error, info};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StageStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
}

// Task 结构体
use std::fmt;

#[derive(Clone)]
pub struct Task {
    pub id: u64,
    pub target_thread_id: usize,
    pub payload: String,
    pub content: Vec<u8>,
    pub retries: u32,
    pub max_retries: u32,
    pub backoff_timer: Option<Instant>,
    pub stages: Vec<Stage>, // Vec 存储不同类型的 Stage
    pub error_message: Option<String>,
    pub shutdown: bool,
}

// 阶段数据结构体
#[derive(Debug, Clone)]
pub struct Stage {
    pub name: String,
    pub status: StageStatus,
    pub start_time: Option<Instant>,
    pub completed_time: Option<Instant>,
    pub result_data: Option<Value>, // 结果数据，在 complete_stage 中设置
    pub error_message: Option<String>,
}

// 实现 Task 相关方法
impl Task {
    pub fn new(id: u64, payload: String, max_retries: u32) -> Self {
        Task {
            id,
            target_thread_id: 0,
            payload,
            content: Vec::new(),
            retries: 0,
            max_retries,
            backoff_timer: None,
            stages: Vec::new(),
            error_message: None,
            shutdown: false,
        }
    }

    // 添加一个新的阶段（不同类型的 result_data）
    pub fn add_stage(&mut self, name: &str) {
        let stage = Stage {
            name: name.to_string(),
            status: StageStatus::InProgress,
            start_time: Some(Instant::now()),
            completed_time: None,
            result_data: None,
            error_message: None,
        };
        self.stages.push(stage);
        crate::metrics::TASK_STATUS
            .with_label_values(&[name, "in_progress"])
            .inc();
    }

    // 完成阶段并设置 result_data
    pub fn complete_stage(&mut self, result_data: Value) {
        if let Some(stage) = self.stages.last_mut() {
            crate::metrics::TASK_STATUS
                .with_label_values(&[&stage.name, "in_progress"])
                .dec();

            stage.status = StageStatus::Completed;
            stage.result_data = Some(result_data); // 设置结果数据
            stage.completed_time = Some(Instant::now());

            // Update metrics
            if let (Some(start), Some(end)) = (stage.start_time, stage.completed_time) {
                let duration = end.duration_since(start).as_secs_f64();
                crate::metrics::TASK_STAGE_DURATION
                    .with_label_values(&[&stage.name])
                    .observe(duration);
            }

            // Update metrics based on stage name
            if stage.name == "inserting" {
                crate::metrics::PROCESSED_TASKS.inc();
            } else if stage.name == "distribution" {
                crate::metrics::SUBMITTED_TASKS.inc();
            }
            let labels = [&stage.name, "completed"];
            crate::metrics::TASK_STATUS.with_label_values(&labels).inc();
        }
    }

    // 失败阶段并设置错误信息
    pub fn fail_stage(&mut self, error_message: String) {
        error!("{}", error_message);
        if let Some(stage) = self.stages.last_mut() {
            crate::metrics::TASK_STATUS
                .with_label_values(&[&stage.name, "in_progress"])
                .dec();
            stage.status = StageStatus::Failed;
            stage.error_message = Some(error_message);
            stage.completed_time = Some(Instant::now());

            // Update metrics
            if let (Some(start), Some(end)) = (stage.start_time, stage.completed_time) {
                let duration = end.duration_since(start).as_secs_f64();
                crate::metrics::TASK_STAGE_DURATION
                    .with_label_values(&[&stage.name])
                    .observe(duration);
            }
            crate::metrics::TASK_STATUS
                .with_label_values(&[&stage.name, "failed"])
                .inc();
            crate::metrics::FAILED_TASKS.inc();
        }
    }

    pub fn pend_stage(&mut self) {
        if let Some(stage) = self.stages.last_mut() {
            stage.status = StageStatus::Pending;
            crate::metrics::TASK_STATUS
                .with_label_values(&[&stage.name, "pending"])
                .inc();
        }
    }

    pub fn process_stage(&mut self) {
        if let Some(stage) = self.stages.last_mut() {
            stage.status = StageStatus::InProgress;
            crate::metrics::TASK_STATUS
                .with_label_values(&[&stage.name, "in_progress"])
                .inc();
        }
    }

    // 获取任务的整体状态（根据栈顶阶段的状态）
    pub fn get_task_status(&self) -> StageStatus {
        if let Some(last_stage) = self.stages.last() {
            last_stage.status.clone()
        } else {
            StageStatus::Pending
        }
    }

    pub fn get_content(&self) -> Option<&[u8]> {
        Some(self.content.as_slice())
    }

    // 获取当前阶段的结果数据（栈顶）
    pub fn get_current_stage_result_data(&self) -> Option<&Value> {
        self.stages.last().and_then(|s| {
            info!("current stage name:{}", s.name);
            s.result_data.as_ref()
        })
    }

    pub fn get_stage_result_data_by_name(&self, stage_name: &str) -> Option<&Value> {
        self.stages
            .iter()
            .find(|s| s.name == stage_name)
            .and_then(|s| {
                // info!("found stage: {}", s.name);
                s.result_data.as_ref()
            })
    }

    // 获取当前阶段的错误信息（栈顶）
    pub fn get_current_stage_error_message(&self) -> Option<&String> {
        self.stages.last().and_then(|s| s.error_message.as_ref())
    }

    pub fn get_id(&self) -> u64 {
        self.id
    }

    pub fn is_failed(&self) -> bool {
        self.get_task_status() == StageStatus::Failed
    }

    pub fn is_completed(&self) -> bool {
        self.get_task_status() == StageStatus::Completed
    }
}
impl Ord for Task {
    fn cmp(&self, other: &Self) -> Ordering {
        other.backoff_timer.cmp(&self.backoff_timer)
    }
}

impl PartialOrd for Task {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Task {
    fn eq(&self, other: &Self) -> bool {
        self.backoff_timer == other.backoff_timer
    }
}

impl Eq for Task {}

impl fmt::Debug for Task {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let content_preview = if self.content.len() > 16 {
            format!(
                "{:?}... ({} bytes)",
                &self.content[..16],
                self.content.len()
            )
        } else {
            format!("{:?}", self.content)
        };

        f.debug_struct("Task")
            .field("id", &self.id)
            .field("target_thread_id", &self.target_thread_id)
            .field("payload", &self.payload)
            .field("content", &content_preview)
            .field("retries", &self.retries)
            .field("max_retries", &self.max_retries)
            .field("backoff_timer", &self.backoff_timer)
            .field("stages", &self.stages)
            .field("error_message", &self.error_message)
            .field("shutdown", &self.shutdown)
            .finish()
    }
}
