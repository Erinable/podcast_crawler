
# **分布式任务调度系统架构文档**

---

## **1. 系统概述**

**目标**：设计并实现一个高并发、可扩展的分布式任务调度系统，支持任务的提交、分发、状态监控以及错误恢复。

### **核心特点**

1. **事件驱动架构**：基于异步消息传递实现模块间解耦。
2. **任务调度**：任务按 Worker 的负载动态分配，支持任务失败后的重试。
3. **实时监控**：支持任务队列状态、Worker 状态、系统性能的实时查询。
4. **水平扩展**：支持动态增加/减少 Worker 节点。

---

## **2. 模块设计与职责**

### **2.1 模块划分**

| 模块名          | 职责                                                                 |
|------------------|----------------------------------------------------------------------|
| **Job Manager**  | 接收任务请求，生成任务 ID，并将任务分发给调度器。                     |
| **Scheduler**    | 管理任务队列，根据 Worker 状态选择节点执行任务。                      |
| **Worker**       | 执行具体任务，并向调度器报告结果；支持失败重试逻辑。                  |
| **State Manager**| 存储任务状态，提供任务状态查询接口，支持持久化操作。                 |
| **Monitoring**   | 提供系统监控功能，包括任务队列状态、Worker 状态等实时数据。          |

---

### **2.2 模块职责细化**

#### **2.2.1 Job Manager**

- **职责**：
  - 接收客户端任务提交。
  - 生成任务唯一标识符。
  - 将任务提交到 Scheduler 的任务队列。
- **关键交互**：
  - 接收外部任务提交请求。
  - 与 Scheduler 通信，将任务添加到调度队列。
  - 与 Monitoring 交互提供任务提交统计信息。

---

#### **2.2.2 Scheduler**

- **职责**：
  - 从任务队列中获取任务。
  - 根据 Worker 的健康状态和负载，选择适合的 Worker。
  - 将任务分配给 Worker。
  - 处理 Worker 返回的结果。
  - 通知 State Manager 更新任务状态。
- **关键交互**：
  - 从 Job Manager 接收任务。
  - 查询 Worker 状态，分配任务。
  - 将结果通知 State Manager。

---

#### **2.2.3 Worker**

- **职责**：
  - 从 Scheduler 接收任务。
  - 执行任务逻辑。
  - 将执行结果反馈给 Scheduler。
  - 支持任务重试机制。
- **关键交互**：
  - 从 Scheduler 接收任务消息。
  - 返回任务执行状态。

---

#### **2.2.4 State Manager**

- **职责**：
  - 存储任务的全局状态。
  - 提供任务状态查询接口。
  - 持久化任务状态（如任务完成、失败等）。
- **关键交互**：
  - 从 Scheduler 接收状态更新。
  - 提供任务状态查询服务。

---

#### **2.2.5 Monitoring**

- **职责**：
  - 提供系统运行时的监控数据（如任务队列长度、Worker 状态等）。
  - 与 State Manager 和 Scheduler 交互获取实时数据。
- **关键交互**：
  - 查询 State Manager 和 Scheduler，获取监控数据。

---

## **3. 接口与消息协议**

### **3.1 消息协议**

定义系统中各模块间的消息类型。

```rust
// 定义消息类型
enum Message {
    // Job Manager -> Scheduler
    SubmitTask {
        task_id: u64,
        payload: String,
    },
    // Scheduler -> Worker
    AssignTask {
        task_id: u64,
        worker_id: u64,
    },
    // Worker -> Scheduler
    TaskResult {
        task_id: u64,
        status: TaskStatus,
    },
    // Scheduler -> State Manager
    UpdateTaskState {
        task_id: u64,
        state: TaskState,
    },
    // Monitoring -> State Manager
    QueryTaskState {
        task_id: u64,
    },
}

// 任务状态
enum TaskState {
    Pending,
    InProgress,
    Completed,
    Failed,
}

// 任务结果状态
enum TaskStatus {
    Success,
    Failure(String), // 包含错误信息
}
```

---

### **3.2 模块间接口**

1. **Job Manager -> Scheduler**
   - **接口描述**：
     - 接口：`submit_task(task_id, payload)`
     - 参数：任务 ID（`task_id`），任务数据（`payload`）。
     - 返回值：无。
   - **调用方式**：异步消息（Tokio mpsc 通道）。

2. **Scheduler -> Worker**
   - **接口描述**：
     - 接口：`assign_task(task_id, worker_id)`
     - 参数：任务 ID（`task_id`），Worker ID（`worker_id`）。
     - 返回值：无。
   - **调用方式**：异步消息（Tokio mpsc 通道）。

3. **Worker -> Scheduler**
   - **接口描述**：
     - 接口：`report_task_result(task_id, status)`
     - 参数：任务 ID（`task_id`），任务状态（`status`）。
     - 返回值：无。
   - **调用方式**：异步消息（Tokio mpsc 通道）。

4. **Scheduler -> State Manager**
   - **接口描述**：
     - 接口：`update_task_state(task_id, state)`
     - 参数：任务 ID（`task_id`），任务状态（`state`）。
     - 返回值：无。
   - **调用方式**：异步消息（Tokio mpsc 通道）。

5. **Monitoring -> State Manager**
   - **接口描述**：
     - 接口：`query_task_state(task_id)`
     - 参数：任务 ID（`task_id`）。
     - 返回值：任务状态（`state`）。
   - **调用方式**：同步查询。

---

## **4. 数据流与控制流**

### **4.1 数据流**

1. **任务提交**：
   - 用户通过 Job Manager 提交任务。
   - 任务信息存入 Scheduler 的任务队列。

2. **任务分配**：
   - Scheduler 根据 Worker 状态选择合适节点。
   - 将任务分配给对应 Worker。

3. **任务状态更新**：
   - Worker 完成任务后通知 Scheduler。
   - Scheduler 更新 State Manager 的任务状态。

---

### **4.2 控制流**

使用时序图描述任务从提交到完成的控制流。

```
Client -> Job Manager: Submit Task
Job Manager -> Scheduler: Add Task to Queue
Scheduler -> Worker: Assign Task
Worker -> Scheduler: Report Task Result
Scheduler -> State Manager: Update Task State
Monitoring -> State Manager: Query Task State
State Manager -> Monitoring: Return State
```

---

## **5. 扩展性与容错设计**

### **5.1 扩展性**

- **水平扩展**：
  - Scheduler 和 Worker 支持动态注册机制，通过配置中心（如 etcd）实现 Worker 节点的动态管理。
- **分布式消息队列**：
  - 使用 Kafka 或 NATS 替代本地消息通道，实现分布式消息传递。

### **5.2 容错性**

1. **Worker 宕机检测**：
   - Scheduler 通过心跳监测 Worker 健康状态。
   - 若 Worker 宕机，重新分配未完成任务。
2. **任务失败重试**：
   - Scheduler 检测任务失败后，将任务重新排入队列。
3. **消息持久化**：
   - 使用持久化存储（如 Redis）记录任务队列状态，保证重启后任务恢复。

---

## **6. 实现工具建议**

1. **编程语言**：Rust。
2. **异步运行时**：Tokio。
3. **消息传递**：Tokio mpsc 通道（本地）；Kafka 或 NATS（分布式）。
4. **持久化**：SQLite（小型部署）；PostgreSQL 或 etcd（分布式部署）。
5. **监控工具**：Prometheus + Grafana。

---
