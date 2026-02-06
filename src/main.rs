// Main entry point for Octaskly distributed task scheduler
// Titik masuk utama untuk Octaskly penjadwal tugas terdistribusi
//
// Supports: dispatcher mode (task scheduling) and worker mode (task execution)
// Mendukung: mode dispatcher (penjadwalan tugas) dan mode worker (eksekusi tugas)

use anyhow::Result;
use octaskly::cmd::Cli;
use octaskly::scheduler::Scheduler;
use octaskly::state::{DispatcherState, WorkerState};
use octaskly::executor::Executor;
use octaskly::protocol::{Message, WorkerInfo};
use octaskly::util;
use std::path::PathBuf;
use std::sync::Arc;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tokio::time::{Duration, interval};
use tokio::sync::RwLock;
use tracing::{error, info, warn, debug};

#[tokio::main]
async fn main() -> Result<()> {
    util::setup_logging();

    let cmd = Cli::parse_and_run()?;

    match cmd {
        octaskly::cmd::Command::Dispatcher {
            bind,
            port,
            workdir,
            ui: _ui,
        } => {
            run_dispatcher(&bind, port, workdir).await?;
        }
        octaskly::cmd::Command::Worker {
            name,
            allow_shell,
            max_jobs,
            ..
        } => {
            run_worker(&name, allow_shell, max_jobs).await?;
        }
        _ => {
            eprintln!("Usage: octaskly <dispatcher | worker | d | w>");
            std::process::exit(1);
        }
    }

    Ok(())
}


async fn run_dispatcher(bind: &str, port: u16, workdir: PathBuf) -> Result<()> {
    // Initialize dispatcher with state management
    // Inisialisasi dispatcher dengan manajemen status
    info!("[DISPATCHER] Starting Octaskly Dispatcher on {}:{}", bind, port);

    let dispatcher_state = Arc::new(DispatcherState::new("dispatcher".to_string(), port));
    let scheduler = Arc::new(Scheduler::new());
    let active_tasks: Arc<RwLock<std::collections::HashMap<String, String>>> = 
        Arc::new(RwLock::new(std::collections::HashMap::new()));

    // Create work directory if not exists
    // Buat direktori kerja jika belum ada
    util::ensure_dir(&workdir).await?;

    // Create network listener on specified address and port
    // Buat listener jaringan pada alamat dan port yang ditentukan
    let addr = format!("{}:{}", bind, port);
    let listener = TcpListener::bind(&addr).await?;
    info!("[DISPATCHER] Listening on {}", addr);

    let listener = Arc::new(listener);

    info!("[DISPATCHER] Ready. Waiting for worker connections...");

    // Spawn task to handle incoming connections from workers
    // Jalankan task untuk menangani koneksi masuk dari worker
    let listener_clone = listener.clone();
    let scheduler_clone = scheduler.clone();
    let dispatcher_state_clone = dispatcher_state.clone();
    let active_tasks_clone = active_tasks.clone();
    
    tokio::spawn(async move {
        loop {
            match listener_clone.accept().await {
                Ok((stream, peer_addr)) => {
                    debug!("[DISPATCHER] Accepted connection from {}", peer_addr);
                    
                    let scheduler = scheduler_clone.clone();
                    let dispatcher_state = dispatcher_state_clone.clone();
                    let active_tasks = active_tasks_clone.clone();
                    
                    tokio::spawn(async move {
                        if let Err(e) = 
                            octaskly::transport::Transport::handle_connection(
                                stream,
                                move |msg| {
                                    let scheduler = scheduler.clone();
                                    let dispatcher_state = dispatcher_state.clone();
                                    let active_tasks = active_tasks.clone();
                                    
                                    Box::pin(async move {
                                        handle_dispatcher_message(
                                            msg,
                                            &scheduler,
                                            &dispatcher_state,
                                            &active_tasks,
                                        )
                                        .await
                                    })
                                }
                            ).await 
                        {
                            error!("Connection handler error: {}", e);
                        }
                    });
                }
                Err(e) => {
                    error!("Accept error: {}", e);
                }
            }
        }
    });

    // Scheduler loop - assign tasks to idle workers
    // Loop penjadwal - tugaskan tugas ke worker yang menganggur
    let scheduler_clone = scheduler.clone();
    let active_tasks_clone = active_tasks.clone();
    
    tokio::spawn(async move {
        let mut interval = interval(Duration::from_millis(500));
        
        loop {
            interval.tick().await;
            
            if let Some((task, mut worker)) = scheduler_clone.schedule_next_task().await {
                debug!("[SCHEDULER] Assigning task {} to worker {}", task.id, worker.id);
                
                // Mark task as assigned
                active_tasks_clone.write().await.insert(task.id.clone(), worker.id.clone());
                
                // Update worker current jobs
                worker.current_jobs += 1;
                scheduler_clone.update_worker(&worker.id, worker.clone()).await;
                
                // Try to send task to worker
                let worker_addr = format!("{}:{}", worker.address, worker.port);
                if let Ok(socket_addr) = worker_addr.parse::<SocketAddr>() {
                    let message = Message::AssignTask(task.clone());
                    if let Err(e) = octaskly::transport::Transport::new().send_message(socket_addr, &message).await {
                        warn!("Failed to send task to worker {}: {}", worker.id, e);
                        // Requeue task
                        scheduler_clone.enqueue(task).await;
                    }
                }
            }
        }
    });

    // Heartbeat cleanup loop - remove offline workers
    // Loop pembersihan detak jantung - hapus worker yang offline
    let scheduler_clone = scheduler.clone();
    
    tokio::spawn(async move {
        let mut interval = interval(Duration::from_secs(10));
        
        loop {
            interval.tick().await;
            scheduler_clone.cleanup_offline_workers(30).await;
        }
    });

    // Handle graceful shutdown
    // Tangani penutupan yang elegan
    tokio::signal::ctrl_c().await?;
    info!("[DISPATCHER] Shutting down gracefully...");

    Ok(())
}

// Handle incoming messages from workers at dispatcher
// Tangani pesan masuk dari worker di dispatcher
async fn handle_dispatcher_message(
    msg: Message,
    scheduler: &Scheduler,
    dispatcher_state: &DispatcherState,
    _active_tasks: &Arc<RwLock<std::collections::HashMap<String, String>>>,
) -> Result<()> {
    match msg {
        // Register worker when it announces itself
        // Daftarkan worker ketika mengumumkan dirinya
        Message::WorkerAnnounce(worker_info) => {
            info!("[DISPATCHER] Worker registered: {} ({}:{})", worker_info.name, worker_info.address, worker_info.port);
            scheduler.register_worker(worker_info).await;
        }
        
        // Task completion notification from worker
        // Notifikasi penyelesaian tugas dari worker
        Message::TaskCompleted(result) => {
            info!("[DISPATCHER] Task {} completed - status: {:?}", result.task_id, result.status);
            dispatcher_state.store_result(result.clone()).await;
            scheduler.worker_job_completed(&result.worker_id).await;
        }
        
        Message::TaskProgress { task_id, progress } => {
            debug!("[DISPATCHER] Task {} progress: {:.1}%", task_id, progress * 100.0);
        }
        
        // Worker heartbeat for health monitoring
        // Detak jantung worker untuk pemantauan kesehatan
        Message::Heartbeat { worker_id, timestamp: _ } => {
            debug!("[DISPATCHER] Heartbeat received from {}", worker_id);
            // Update worker last_heartbeat in scheduler
            // Perbarui last_heartbeat worker di penjadwal
            let workers = scheduler.get_workers().await;
            if let Some(mut worker) = workers.iter().find(|w| w.id == worker_id).cloned() {
                worker.last_heartbeat = chrono::Local::now().timestamp();
                scheduler.update_worker(&worker_id, worker).await;
            }
        }
        
        _ => {
            warn!("Unexpected message type: {:?}", msg);
        }
    }
    
    Ok(())
}

// Worker process initialization and main loop
// Inisialisasi proses worker dan loop utama
async fn run_worker(name: &str, allow_shell: bool, max_jobs: usize) -> Result<()> {
    info!("[WORKER] Starting Worker '{}' with max_jobs={}", name, max_jobs);

    let local_ip = util::get_local_ip().unwrap_or_else(|| "127.0.0.1".to_string());
    let port = find_available_port(7879).await?;

    let worker_state = Arc::new(WorkerState::new(name.to_string(), port));
    let executor = Arc::new(Executor::new(PathBuf::from("./work"), allow_shell));

    let worker_info = WorkerInfo::new(
        name.to_string(),
        local_ip.clone(),
        port,
        max_jobs,
    );

    info!(
        "[WORKER] Registered at {}:{}",
        local_ip, port
    );
    info!("[WORKER] Waiting for dispatcher assignment...");

    // Start listening for incoming connections from dispatcher
    // Mulai mendengarkan koneksi masuk dari dispatcher
    let addr = format!("0.0.0.0:{}", port);
    let listener = TcpListener::bind(&addr).await?;
    info!("[WORKER] Listening on {}", addr);

    let listener = Arc::new(listener);
    let worker_info_announced = Arc::new(RwLock::new(false));
    let worker_info_to_announce = worker_info.clone();

    // Spawn connection handler task
    // Jalankan task penanganan koneksi
    let listener_clone = listener.clone();
    let worker_state_clone = worker_state.clone();
    let executor_clone = executor.clone();
    let worker_info_announced_clone = worker_info_announced.clone();
    let worker_info_for_handler = worker_info_to_announce.clone();
    
    tokio::spawn(async move {
        loop {
            match listener_clone.accept().await {
                Ok((stream, peer_addr)) => {
                    debug!("[WORKER] Connection established with dispatcher at {}", peer_addr);
                    
                    // Announce worker to dispatcher if not already done
                    // Umumkan worker ke dispatcher jika belum dilakukan
                    if !*worker_info_announced_clone.read().await {
                        *worker_info_announced_clone.write().await = true;
                        let announce_msg = Message::WorkerAnnounce(worker_info_for_handler.clone());
                        let announce_addr = peer_addr;
                        
                        if let Err(e) = octaskly::transport::Transport::new().send_message(announce_addr, &announce_msg).await {
                            warn!("Failed to announce worker: {}", e);
                        }
                    }
                    
                    let worker_state = worker_state_clone.clone();
                    let executor = executor_clone.clone();
                    
                    tokio::spawn(async move {
                        if let Err(e) = 
                            octaskly::transport::Transport::handle_connection(
                                stream,
                                move |msg| {
                                    let worker_state = worker_state.clone();
                                    let executor = executor.clone();
                                    
                                    Box::pin(async move {
                                        handle_worker_message(msg, &worker_state, &executor, peer_addr).await
                                    })
                                }
                            ).await 
                        {
                            error!("Worker connection handler error: {}", e);
                        }
                    });
                }
                Err(e) => {
                    error!("Accept error: {}", e);
                }
            }
        }
    });

    // Heartbeat loop - send periodic heartbeats to dispatcher
    // Loop detak jantung - kirim detak jantung berkala ke dispatcher
    tokio::spawn(async move {
        let mut interval = interval(Duration::from_secs(5));
        
        loop {
            interval.tick().await;
            debug!("[WORKER] Heartbeat check");
            // Note: In production, we'd track dispatcher address and send heartbeat
            // Catatan: Dalam produksi, kami akan melacak alamat dispatcher dan mengirim heartbeat
        }
    });

    // Keep running
    // Tetap berjalan
    tokio::signal::ctrl_c().await?;
    info!("[WORKER] Shutting down gracefully...");
    
    Ok(())
}

// Handle task execution messages on worker
// Tangani pesan eksekusi tugas di worker
async fn handle_worker_message(
    msg: Message,
    worker_state: &WorkerState,
    executor: &Executor,
    dispatcher_addr: SocketAddr,
) -> Result<()> {
    match msg {
        // Execute assigned task from dispatcher
        // Jalankan tugas yang ditugaskan dari dispatcher
        Message::AssignTask(task) => {
            info!("[WORKER] Task received for execution: {}", task.id);
            
            let task_id = task.id.clone();
            worker_state.set_current_task(Some(task.clone())).await;
            
            // Execute task with timeout protection
            // Jalankan tugas dengan perlindungan timeout
            match executor.execute_with_timeout(&task).await {
                Ok(result) => {
                    info!("[WORKER] Task {} execution completed successfully", task_id);
                    
                    let task_result = octaskly::protocol::TaskResult {
                        task_id: task_id.clone(),
                        worker_id: "unknown".to_string(),
                        status: result.status,
                        stdout: result.stdout,
                        stderr: result.stderr,
                        exit_code: result.exit_code,
                        duration_ms: result.duration_ms,
                        completed_at: chrono::Local::now().timestamp(),
                    };
                    
                    // Send result back to dispatcher
                    // Kirim hasil kembali ke dispatcher
                    let result_msg = Message::TaskCompleted(task_result);
                    if let Err(e) = octaskly::transport::Transport::new().send_message(dispatcher_addr, &result_msg).await {
                        error!("[WORKER] Failed to send task result: {}", e);
                    }
                    
                    worker_state.set_current_task(None).await;
                }
                Err(e) => {
                    error!("Task execution failed: {}", e);
                    worker_state.set_current_task(None).await;
                }
            }
        }
        
        // Task cancellation request
        // Permintaan pembatalan tugas
        Message::CancelTask { task_id } => {
            info!("[WORKER] Cancel request received for task: {}", task_id);
            worker_state.set_current_task(None).await;
        }
        
        _ => {
            warn!("Unexpected message type for worker: {:?}", msg);
        }
    }
    
    Ok(())
}

/// Find an available port starting from the given port
/// Cari port yang tersedia dimulai dari port yang diberikan
async fn find_available_port(start_port: u16) -> Result<u16> {
    for port in start_port..(start_port + 100) {
        let addr = format!("0.0.0.0:{}", port);
        match TcpListener::bind(&addr).await {
            Ok(_) => return Ok(port),
            Err(_) => continue,
        }
    }
    Err(anyhow::anyhow!("No available port found"))
}

