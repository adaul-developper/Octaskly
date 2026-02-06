// Example: Using OCTASKLY as a library
use octaskly::protocol::Task;
use octaskly::scheduler::Scheduler;
use octaskly::executor::Executor;
use std::path::PathBuf;
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Example 1: Create and manage tasks
    println!("=== OCTASKLY Library Example ===\n");

    println!("📋 Example 1: Creating tasks");
    let task1 = Task::new("echo 'Task 1 completed'".to_string());
    let task2 = Task::new("ls -la".to_string());
    println!("Created task: {} - {}", task1.id, task1.command);
    println!("Created task: {} - {}", task2.id, task2.command);

    // Example 2: Using scheduler to queue tasks
    println!("\n📋 Example 2: Queuing tasks with scheduler");
    let scheduler = Arc::new(Scheduler::new());
    scheduler.enqueue(task1).await;
    scheduler.enqueue(task2).await;
    println!("Queue size: {}", scheduler.queue_size().await);

    // Example 3: Execute a task directly
    println!("\n📋 Example 3: Executing task directly");
    let executor = Executor::new(PathBuf::from("/tmp"), true);
    let task = Task::new("echo 'Hello from OCTASKLY'".to_string());
    
    match executor.execute_with_timeout(&task).await {
        Ok(result) => {
            println!("✅ Task executed!");
            println!("   Status: {:?}", result.status);
            println!("   Output: {}", result.stdout);
            println!("   Duration: {}ms", result.duration_ms);
        }
        Err(e) => {
            println!("❌ Error: {}", e);
        }
    }

    // Example 4: Use protocol messages
    println!("\n📋 Example 4: Protocol messages");
    use octaskly::protocol::{Message, WorkerInfo};
    
    let worker = WorkerInfo::new(
        "example-worker".to_string(),
        "192.168.1.100".to_string(),
        7879,
        4,
    );
    
    let msg = Message::WorkerAnnounce(worker.clone());
    println!("✅ Created message: {:?}", msg);

    // Example 5: Validate commands
    println!("\n📋 Example 5: Command validation");
    let executor = Executor::new(PathBuf::from("/tmp"), true);
    
    let safe_cmd = "echo 'safe command'";
    let dangerous_cmd = "rm -rf /";
    
    println!("Validating '{}': {}", safe_cmd, executor.validate_command(safe_cmd));
    println!("Validating '{}': {}", dangerous_cmd, executor.validate_command(dangerous_cmd));

    println!("\n✨ Examples completed!");
    Ok(())
}
