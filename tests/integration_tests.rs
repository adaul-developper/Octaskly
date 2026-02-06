// Integration tests for OCTASKLY
#[cfg(test)]
mod integration_tests {
    use octaskly::protocol::{Task, WorkerInfo, TaskStatus};
    use octaskly::scheduler::Scheduler;
    use octaskly::executor::Executor;
    use octaskly::state::{DispatcherState, WorkerState};
    use std::path::PathBuf;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_dispatcher_worker_workflow() {
        // Setup dispatcher
        let _dispatcher = Arc::new(DispatcherState::new("test-dispatcher".to_string(), 7878));
        let scheduler = Arc::new(Scheduler::new());

        // Setup worker info
        let worker = WorkerInfo::new(
            "test-worker".to_string(),
            "127.0.0.1".to_string(),
            7879,
            2,
        );

        // Register worker with scheduler
        scheduler.register_worker(worker.clone()).await;
        let workers = scheduler.get_workers().await;
        assert_eq!(workers.len(), 1);
        assert_eq!(workers[0].name, "test-worker");
    }

    #[tokio::test]
    async fn test_task_execution() {
        let executor = Executor::new(PathBuf::from("/tmp"), true);
        let task = Task::new("echo 'test'".to_string());
        
        let result = executor.execute_with_timeout(&task).await;
        assert!(result.is_ok());
        
        let result = result.unwrap();
        assert_eq!(result.status, TaskStatus::Completed);
        assert!(result.stdout.contains("test"));
    }

    #[tokio::test]
    async fn test_task_timeout() {
        let executor = Executor::new(PathBuf::from("/tmp"), true);
        let mut task = Task::new("sleep 10".to_string());
        task.timeout = 1; // 1 second timeout
        
        let result = executor.execute_with_timeout(&task).await;
        assert!(result.is_ok());
        
        let result = result.unwrap();
        assert_eq!(result.status, TaskStatus::TimedOut);
    }

    #[tokio::test]
    async fn test_task_queue_fifo() {
        let scheduler = Arc::new(Scheduler::new());
        
        let task1 = Task::new("echo '1'".to_string());
        let task2 = Task::new("echo '2'".to_string());
        let id1 = task1.id.clone();
        let id2 = task2.id.clone();
        
        scheduler.enqueue(task1).await;
        scheduler.enqueue(task2).await;
        
        assert_eq!(scheduler.queue_size().await, 2);
        
        let dequeued1 = scheduler.dequeue().await;
        assert!(dequeued1.is_some());
        assert_eq!(dequeued1.unwrap().id, id1);
        
        let dequeued2 = scheduler.dequeue().await;
        assert!(dequeued2.is_some());
        assert_eq!(dequeued2.unwrap().id, id2);
    }

    #[tokio::test]
    async fn test_worker_state_management() {
        let worker_state = Arc::new(WorkerState::new("test-worker".to_string(), 7879));
        
        // Initially no task
        assert!(worker_state.get_current_task().await.is_none());
        
        let task = Task::new("test command".to_string());
        worker_state.set_current_task(Some(task.clone())).await;
        
        let current = worker_state.get_current_task().await;
        assert!(current.is_some());
        assert_eq!(current.unwrap().id, task.id);
        
        // Clear task
        worker_state.set_current_task(None).await;
        assert!(worker_state.get_current_task().await.is_none());
    }

    #[tokio::test]
    async fn test_dispatcher_state_results() {
        let dispatcher = Arc::new(DispatcherState::new("test-dispatcher".to_string(), 7878));
        
        assert_eq!(dispatcher.get_history_count().await, 0);
        
        // Simulate task completion
        let result = octaskly::protocol::TaskResult {
            task_id: "task-1".to_string(),
            worker_id: "worker-1".to_string(),
            status: TaskStatus::Completed,
            stdout: "test output".to_string(),
            stderr: String::new(),
            exit_code: Some(0),
            duration_ms: 100,
            completed_at: chrono::Local::now().timestamp(),
        };
        
        dispatcher.store_result(result.clone()).await;
        assert_eq!(dispatcher.get_history_count().await, 1);
        
        let stored = dispatcher.get_result("task-1").await;
        assert!(stored.is_some());
        assert_eq!(stored.unwrap().task_id, "task-1");
    }

    #[test]
    fn test_worker_idle_state() {
        let mut worker = WorkerInfo::new(
            "test".to_string(),
            "127.0.0.1".to_string(),
            7879,
            2,
        );
        
        assert!(worker.is_idle()); // current_jobs = 0, max_jobs = 2
        
        worker.current_jobs = 2;
        assert!(!worker.is_idle());
        
        worker.current_jobs = 1;
        assert!(worker.is_idle());
    }

    #[test]
    fn test_command_validation() {
        let executor = Executor::new(PathBuf::from("/tmp"), true);
        
        // Safe commands
        assert!(executor.validate_command("echo 'hello'"));
        assert!(executor.validate_command("ls -la"));
        assert!(executor.validate_command("cargo build"));
        
        // Dangerous commands
        assert!(!executor.validate_command("rm -rf /"));
        assert!(!executor.validate_command("dd if=/dev/zero"));
        assert!(!executor.validate_command(":(){:|:&};:"));
    }
}
