use crate::protocol::{Task, TaskStatus};
use anyhow::Result;
use std::path::PathBuf;
use std::process::Stdio;
use tokio::process::Command;
use tracing::{error, info};

// Task execution engine for running shell commands
// Mesin eksekusi tugas untuk menjalankan perintah shell
pub struct Executor {
    workdir: PathBuf,
    allow_shell: bool,
}

impl Executor {
    // Create new executor instance with working directory and permissions
    // Buat instance executor baru dengan direktori kerja dan izin
    pub fn new(workdir: PathBuf, allow_shell: bool) -> Self {
        Self {
            workdir,
            allow_shell,
        }
    }

    // Execute task asynchronously with output capture
    // Jalankan tugas secara asinkron dengan penangkapan output
    pub async fn execute(&self, task: &Task) -> Result<ExecutionResult> {
        if !self.allow_shell {
            return Err(anyhow::anyhow!("Shell execution is not allowed"));
        }

        info!("Executing task {}: {}", task.id, task.command);

        let start_time = std::time::Instant::now();

        // Create working directory if needed
        // Buat direktori kerja jika diperlukan
        tokio::fs::create_dir_all(&self.workdir).await.ok();

        let mut child = Command::new("sh")
            .arg("-c")
            .arg(&task.command)
            .current_dir(&self.workdir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let mut stdout = String::new();
        let mut stderr = String::new();

        if let Some(mut out) = child.stdout.take() {
            tokio::io::AsyncReadExt::read_to_string(&mut out, &mut stdout).await.ok();
        }

        if let Some(mut err) = child.stderr.take() {
            tokio::io::AsyncReadExt::read_to_string(&mut err, &mut stderr).await.ok();
        }

        let status = child.wait().await?;
        let exit_code = status.code();
        let duration_ms = start_time.elapsed().as_millis() as u64;

        if exit_code == Some(0) {
            info!("Task {} completed successfully in {}ms", task.id, duration_ms);
        } else {
            error!(
                "Task {} failed with exit code {:?}",
                task.id, exit_code
            );
        }

        Ok(ExecutionResult {
            task_id: task.id.clone(),
            status: if exit_code == Some(0) {
                TaskStatus::Completed
            } else {
                TaskStatus::Failed
            },
            stdout,
            stderr,
            exit_code,
            duration_ms,
        })
    }

    // Execute task with timeout protection to prevent infinite runs
    // Jalankan tugas dengan perlindungan timeout untuk mencegah proses tak terbatas
    pub async fn execute_with_timeout(&self, task: &Task) -> Result<ExecutionResult> {
        let timeout_duration = std::time::Duration::from_secs(task.timeout);
        
        match tokio::time::timeout(timeout_duration, self.execute(task)).await {
            Ok(result) => result,
            Err(_) => {
                error!("Task {} timed out after {}s", task.id, task.timeout);
                Ok(ExecutionResult {
                    task_id: task.id.clone(),
                    status: TaskStatus::TimedOut,
                    stdout: String::new(),
                    stderr: format!("Task timed out after {} seconds", task.timeout),
                    exit_code: None,
                    duration_ms: task.timeout * 1000,
                })
            }
        }
    }

    // Validate command for dangerous patterns before execution
    // Validasi perintah untuk pola berbahaya sebelum eksekusi
    pub fn validate_command(&self, command: &str) -> bool {
        if !self.allow_shell {
            return false;
        }

        // Security check: disallow dangerous system patterns
        // Pengecekan keamanan: tolak pola sistem berbahaya
        let dangerous_patterns = [
            "rm -rf /",
            "dd if=/dev/zero",
            ":(){:|:&};:",
        ];

        for pattern in &dangerous_patterns {
            if command.contains(pattern) {
                return false;
            }
        }

        true
    }
}

// Result structure returned after task execution
// Struktur hasil yang dikembalikan setelah eksekusi tugas
pub struct ExecutionResult {
    pub task_id: String,
    pub status: TaskStatus,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i32>,
    pub duration_ms: u64,
}

// Unit tests for executor validation
// Tes unit untuk validasi executor
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_command() {
        let executor = Executor::new(PathBuf::from("/tmp"), true);

        assert!(executor.validate_command("echo hello"));
        assert!(executor.validate_command("ls -la"));
        assert!(!executor.validate_command("rm -rf /"));
    }
}
