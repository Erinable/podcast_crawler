use serde_json::Value;
use std::cmp::Ordering;
use std::fmt::Debug;
use std::time::Instant;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StageStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
}

// Task 结构体
#[derive(Debug, Clone)]
pub struct Task {
    pub id: u64,
    pub target_thread_id: usize,
    pub payload: String,
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
    }

    // 完成阶段并设置 result_data
    pub fn complete_stage(&mut self, result_data: Value) {
        if let Some(stage) = self.stages.last_mut() {
            stage.status = StageStatus::Completed;
            stage.result_data = Some(result_data); // 设置结果数据
            stage.completed_time = Some(Instant::now());
        }
    }

    // 失败阶段并设置错误信息
    pub fn fail_stage(&mut self, error_message: String) {
        if let Some(stage) = self.stages.last_mut() {
            stage.status = StageStatus::Failed;
            stage.error_message = Some(error_message);
            stage.completed_time = Some(Instant::now());
        }
    }

    pub fn pend_stage(&mut self) {
        if let Some(stage) = self.stages.last_mut() {
            stage.status = StageStatus::Pending;
        }
    }

    pub fn process_stage(&mut self) {
        if let Some(stage) = self.stages.last_mut() {
            stage.status = StageStatus::InProgress;
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

    // 获取当前阶段的结果数据（栈顶）
    pub fn get_current_stage_result_data(&self) -> Option<&Value> {
        self.stages.last().and_then(|s| s.result_data.as_ref())
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
