use crate::protocol::{Task, TaskResult};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::debug;

// Dispatcher state container for managing tasks and workers
// Kontainer status dispatcher untuk mengelola tugas dan worker
pub struct DispatcherState {
    pub id: String,
    pub name: String,
    pub port: u16,
    pub task_results: Arc<RwLock<HashMap<String, TaskResult>>>,
    pub completed_tasks: Arc<RwLock<Vec<Task>>>,
}

impl DispatcherState {
    // Initialize new dispatcher with unique identifier
    // Inisialisasi dispatcher baru dengan pengenal unik
    pub fn new(name: String, port: u16) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            port,
            task_results: Arc::new(RwLock::new(HashMap::new())),
            completed_tasks: Arc::new(RwLock::new(Vec::new())),
        }
    }

    // Store task execution result
    // Simpan hasil eksekusi tugas
    pub async fn store_result(&self, result: TaskResult) {
        debug!("Storing result for task {}", result.task_id);
        self.task_results
            .write()
            .await
            .insert(result.task_id.clone(), result);
    }

    // Retrieve result for specific task
    // Ambil hasil untuk tugas tertentu
    pub async fn get_result(&self, task_id: &str) -> Option<TaskResult> {
        self.task_results.read().await.get(task_id).cloned()
    }

    // Record completed task in history
    // Catat tugas selesai dalam riwayat
    pub async fn add_completed_task(&self, task: Task) {
        self.completed_tasks.write().await.push(task);
    }

    // Retrieve all completed tasks
    // Ambil semua tugas selesai
    pub async fn get_completed_tasks(&self) -> Vec<Task> {
        self.completed_tasks.read().await.clone()
    }

    // Get number of results stored
    // Dapatkan jumlah hasil yang disimpan
    pub async fn get_history_count(&self) -> usize {
        self.task_results.read().await.len()
    }
}

/// Worker state
pub struct WorkerState {
    pub id: String,
    pub name: String,
    pub port: u16,
    pub current_task: Arc<RwLock<Option<Task>>>,
    pub completed_tasks: Arc<RwLock<Vec<TaskResult>>>,
}

impl WorkerState {
    pub fn new(name: String, port: u16) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            port,
            current_task: Arc::new(RwLock::new(None)),
            completed_tasks: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn set_current_task(&self, task: Option<Task>) {
        *self.current_task.write().await = task;
    }

    pub async fn get_current_task(&self) -> Option<Task> {
        self.current_task.read().await.clone()
    }

    pub async fn add_completed_task(&self, result: TaskResult) {
        self.completed_tasks.write().await.push(result);
    }

    pub async fn get_completed_count(&self) -> usize {
        self.completed_tasks.read().await.len()
    }

    pub async fn get_completed_tasks(&self) -> Vec<TaskResult> {
        self.completed_tasks.read().await.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_dispatcher_state() {
        let dispatcher = DispatcherState::new("dispatcher-1".to_string(), 7878);
        assert_eq!(dispatcher.get_history_count().await, 0);
    }

    #[tokio::test]
    async fn test_worker_state() {
        let worker = WorkerState::new("worker-1".to_string(), 7879);
        assert!(worker.get_current_task().await.is_none());
    }
}
