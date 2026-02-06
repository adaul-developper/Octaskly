use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Represents a compute task to be executed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub command: String,
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
    pub timeout: u64,
    pub env: HashMap<String, String>,
    pub created_at: i64,
}

impl Task {
    pub fn new(command: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            command,
            inputs: Vec::new(),
            outputs: Vec::new(),
            timeout: 600, // 10 minutes default
            env: HashMap::new(),
            created_at: chrono::Local::now().timestamp(),
        }
    }
}

/// Represents the result of task execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    pub task_id: String,
    pub worker_id: String,
    pub status: TaskStatus,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i32>,
    pub duration_ms: u64,
    pub completed_at: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
    TimedOut,
}

/// Worker information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerInfo {
    pub id: String,
    pub name: String,
    pub address: String,
    pub port: u16,
    pub max_jobs: usize,
    pub current_jobs: usize,
    pub allow_shell: bool,
    pub last_heartbeat: i64,
    pub platform: String,
}

impl WorkerInfo {
    pub fn new(name: String, address: String, port: u16, max_jobs: usize) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            address,
            port,
            max_jobs,
            current_jobs: 0,
            allow_shell: true,
            last_heartbeat: chrono::Local::now().timestamp(),
            platform: std::env::consts::OS.to_string(),
        }
    }

    pub fn is_idle(&self) -> bool {
        self.current_jobs < self.max_jobs
    }
}

/// Protocol messages for communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Message {
    /// Worker announces itself to dispatcher
    WorkerAnnounce(WorkerInfo),
    
    /// Dispatcher assigns a task to worker
    AssignTask(Task),
    
    /// Worker reports task progress
    TaskProgress {
        task_id: String,
        progress: f32,
    },
    
    /// Worker reports task completion
    TaskCompleted(TaskResult),
    
    /// Heartbeat message
    Heartbeat {
        worker_id: String,
        timestamp: i64,
    },
    
    /// Cancel a task
    CancelTask {
        task_id: String,
    },
    
    /// Acknowledge message
    Ack {
        message_id: String,
    },
}
