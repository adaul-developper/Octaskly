use crate::protocol::{Task, WorkerInfo};
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};

// Task scheduler managing queue and worker assignment
// Penjadwal tugas mengelola antrian dan penugasan worker
pub struct Scheduler {
    queue: Arc<RwLock<VecDeque<Task>>>,
    workers: Arc<RwLock<Vec<WorkerInfo>>>,
}

impl Scheduler {
    // Initialize new scheduler with empty queue and workers
    // Inisialisasi penjadwal baru dengan antrian dan worker kosong
    pub fn new() -> Self {
        Self {
            queue: Arc::new(RwLock::new(VecDeque::new())),
            workers: Arc::new(RwLock::new(Vec::new())),
        }
    }

    // Add task to the work queue for distribution
    // Tambahkan tugas ke antrian kerja untuk distribusi
    pub async fn enqueue(&self, task: Task) {
        info!("Enqueued task {}: {}", task.id, task.command);
        self.queue.write().await.push_back(task);
    }

    // Remove and return first task from queue (FIFO)
    // Hapus dan kembalikan tugas pertama dari antrian (FIFO)
    pub async fn dequeue(&self) -> Option<Task> {
        self.queue.write().await.pop_front()
    }

    // Get current number of pending tasks
    // Dapatkan jumlah tugas yang tertunda saat ini
    pub async fn queue_size(&self) -> usize {
        self.queue.read().await.len()
    }

    // Register new worker with scheduler
    // Daftarkan worker baru dengan penjadwal
    pub async fn register_worker(&self, worker: WorkerInfo) {
        debug!("Registering worker: {}", worker.name);
        self.workers.write().await.push(worker);
    }

    // Update worker information
    // Perbarui informasi worker
    pub async fn update_worker(&self, worker_id: &str, worker: WorkerInfo) {
        let mut workers = self.workers.write().await;
        if let Some(pos) = workers.iter().position(|w| w.id == worker_id) {
            workers[pos] = worker;
        }
    }

    // Decrement worker job count on task completion
    // Kurangi jumlah pekerjaan worker saat tugas selesai
    pub async fn worker_job_completed(&self, worker_id: &str) {
        let mut workers = self.workers.write().await;
        if let Some(worker) = workers.iter_mut().find(|w| w.id == worker_id) {
            if worker.current_jobs > 0 {
                worker.current_jobs -= 1;
            }
        }
    }

    // Find first idle worker ready to accept tasks
    // Temukan worker menganggur pertama yang siap menerima tugas
    pub async fn get_idle_worker(&self) -> Option<WorkerInfo> {
        let workers = self.workers.read().await;
        workers
            .iter()
            .find(|w| w.is_idle())
            .cloned()
    }

    // Get all workers below job capacity
    // Dapatkan semua worker di bawah kapasitas pekerjaan
    pub async fn get_idle_workers(&self) -> Vec<WorkerInfo> {
        let workers = self.workers.read().await;
        workers.iter().filter(|w| w.is_idle()).cloned().collect()
    }

    // Retrieve list of all registered workers
    // Ambil daftar semua worker yang terdaftar
    pub async fn get_workers(&self) -> Vec<WorkerInfo> {
        self.workers.read().await.clone()
    }

    // Remove inactive workers based on heartbeat timeout
    // Hapus worker tidak aktif berdasarkan timeout detak jantung
    pub async fn cleanup_offline_workers(&self, heartbeat_timeout_secs: i64) {
        let now = chrono::Local::now().timestamp();
        let mut workers = self.workers.write().await;
        
        let initial_count = workers.len();
        workers.retain(|w| (now - w.last_heartbeat) < heartbeat_timeout_secs);
        
        let removed = initial_count - workers.len();
        if removed > 0 {
            info!("Removed {} offline workers", removed);
        }
    }

    // Schedule next task using FIFO algorithm
    // Jadwalkan tugas berikutnya menggunakan algoritma FIFO
    pub async fn schedule_next_task(&self) -> Option<(Task, WorkerInfo)> {
        // Get next task from queue
        // Dapatkan tugas berikutnya dari antrian
        if let Some(task) = self.dequeue().await {
            // Find an idle worker
            // Temukan worker menganggur
            if let Some(mut worker) = self.get_idle_worker().await {
                worker.current_jobs += 1;
                self.update_worker(&worker.id, worker.clone()).await;
                info!("Scheduled task {} to worker {}", task.id, worker.name);
                return Some((task, worker));
            } else {
                // Re-queue the task if no worker available
                // Masukkan kembali tugas jika tidak ada worker tersedia
                self.enqueue(task).await;
            }
        }

        None
    }
}

impl Default for Scheduler {
    fn default() -> Self {
        Self::new()
    }
}

// Unit tests for scheduler functionality
// Tes unit untuk fungsionalitas penjadwal
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_scheduler_enqueue_dequeue() {
        let scheduler = Scheduler::new();
        let task = Task::new("echo hello".to_string());
        let task_id = task.id.clone();

        scheduler.enqueue(task).await;
        assert_eq!(scheduler.queue_size().await, 1);

        let dequeued = scheduler.dequeue().await;
        assert!(dequeued.is_some());
        assert_eq!(dequeued.unwrap().id, task_id);
    }
}
