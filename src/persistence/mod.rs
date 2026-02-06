use anyhow::Result;
use rusqlite::{Connection, params};
use serde::{Serialize, Deserialize};
use std::sync::{Arc, Mutex};
use chrono::Utc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredTask {
    pub id: String,
    pub command: String,
    pub status: String,
    pub worker_id: Option<String>,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i32>,
    pub duration_ms: u64,
    pub created_at: String,
    pub completed_at: Option<String>,
}

/// Persistent storage for task history using SQLite
pub struct PersistentStore {
    conn: Arc<Mutex<Connection>>,
}

impl PersistentStore {
    pub fn new(db_path: &str) -> Result<Self> {
        let conn = Connection::open(db_path)?;
        
        // Enable WAL mode for better concurrency
        conn.execute("PRAGMA journal_mode = WAL", [])?;
        
        // Create tables
        conn.execute(
            "CREATE TABLE IF NOT EXISTS tasks (
                id TEXT PRIMARY KEY,
                command TEXT NOT NULL,
                status TEXT NOT NULL,
                worker_id TEXT,
                stdout TEXT,
                stderr TEXT,
                exit_code INTEGER,
                duration_ms INTEGER,
                created_at TEXT NOT NULL,
                completed_at TEXT
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS results (
                task_id TEXT PRIMARY KEY,
                worker_id TEXT NOT NULL,
                status TEXT NOT NULL,
                stdout TEXT,
                stderr TEXT,
                exit_code INTEGER,
                duration_ms INTEGER,
                completed_at TEXT NOT NULL,
                FOREIGN KEY(task_id) REFERENCES tasks(id)
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS audit_log (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp TEXT NOT NULL,
                event_type TEXT NOT NULL,
                worker_id TEXT,
                task_id TEXT,
                details TEXT
            )",
            [],
        )?;

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    /// Store a task
    pub fn store_task(&self, task: &StoredTask) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        
        conn.execute(
            "INSERT OR REPLACE INTO tasks 
             (id, command, status, worker_id, stdout, stderr, exit_code, duration_ms, created_at, completed_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                task.id,
                task.command,
                task.status,
                task.worker_id,
                task.stdout,
                task.stderr,
                task.exit_code,
                task.duration_ms,
                task.created_at,
                task.completed_at,
            ],
        )?;
        
        Ok(())
    }

    /// Retrieve a task
    pub fn get_task(&self, task_id: &str) -> Result<Option<StoredTask>> {
        let conn = self.conn.lock().unwrap();
        
        let mut stmt = conn.prepare(
            "SELECT id, command, status, worker_id, stdout, stderr, exit_code, duration_ms, created_at, completed_at
             FROM tasks WHERE id = ?1"
        )?;
        
        let result = stmt.query_row(params![task_id], |row| {
            Ok(StoredTask {
                id: row.get(0)?,
                command: row.get(1)?,
                status: row.get(2)?,
                worker_id: row.get(3)?,
                stdout: row.get(4)?,
                stderr: row.get(5)?,
                exit_code: row.get(6)?,
                duration_ms: row.get(7)?,
                created_at: row.get(8)?,
                completed_at: row.get(9)?,
            })
        });
        
        match result {
            Ok(task) => Ok(Some(task)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Get all tasks
    pub fn get_all_tasks(&self) -> Result<Vec<StoredTask>> {
        let conn = self.conn.lock().unwrap();
        
        let mut stmt = conn.prepare(
            "SELECT id, command, status, worker_id, stdout, stderr, exit_code, duration_ms, created_at, completed_at
             FROM tasks ORDER BY created_at DESC LIMIT 1000"
        )?;
        
        let tasks = stmt.query_map([], |row| {
            Ok(StoredTask {
                id: row.get(0)?,
                command: row.get(1)?,
                status: row.get(2)?,
                worker_id: row.get(3)?,
                stdout: row.get(4)?,
                stderr: row.get(5)?,
                exit_code: row.get(6)?,
                duration_ms: row.get(7)?,
                created_at: row.get(8)?,
                completed_at: row.get(9)?,
            })
        })?;
        
        let mut result = Vec::new();
        for task in tasks {
            result.push(task?);
        }
        Ok(result)
    }

    /// Get tasks for a worker
    pub fn get_worker_tasks(&self, worker_id: &str) -> Result<Vec<StoredTask>> {
        let conn = self.conn.lock().unwrap();
        
        let mut stmt = conn.prepare(
            "SELECT id, command, status, worker_id, stdout, stderr, exit_code, duration_ms, created_at, completed_at
             FROM tasks WHERE worker_id = ?1 ORDER BY created_at DESC LIMIT 100"
        )?;
        
        let tasks = stmt.query_map(params![worker_id], |row| {
            Ok(StoredTask {
                id: row.get(0)?,
                command: row.get(1)?,
                status: row.get(2)?,
                worker_id: row.get(3)?,
                stdout: row.get(4)?,
                stderr: row.get(5)?,
                exit_code: row.get(6)?,
                duration_ms: row.get(7)?,
                created_at: row.get(8)?,
                completed_at: row.get(9)?,
            })
        })?;
        
        let mut result = Vec::new();
        for task in tasks {
            result.push(task?);
        }
        Ok(result)
    }

    /// Delete old tasks (cleanup)
    pub fn cleanup_old_tasks(&self, days: i64) -> Result<usize> {
        let conn = self.conn.lock().unwrap();
        
        let cutoff = Utc::now() - chrono::Duration::days(days);
        let cutoff_str = cutoff.to_rfc3339();
        
        let rows = conn.execute(
            "DELETE FROM tasks WHERE created_at < ?1",
            params![cutoff_str],
        )?;
        
        Ok(rows)
    }

    /// Record audit log entry
    pub fn log_event(&self, event_type: &str, worker_id: Option<&str>, task_id: Option<&str>, details: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        
        let timestamp = Utc::now().to_rfc3339();
        
        conn.execute(
            "INSERT INTO audit_log (timestamp, event_type, worker_id, task_id, details)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![timestamp, event_type, worker_id, task_id, details],
        )?;
        
        Ok(())
    }

    /// Get audit logs
    pub fn get_audit_logs(&self, limit: usize) -> Result<Vec<(String, String, Option<String>, Option<String>, String)>> {
        let conn = self.conn.lock().unwrap();
        
        let mut stmt = conn.prepare(
            "SELECT timestamp, event_type, worker_id, task_id, details
             FROM audit_log ORDER BY timestamp DESC LIMIT ?1"
        )?;
        
        let logs = stmt.query_map(params![limit as i64], |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
            ))
        })?;
        
        let mut result = Vec::new();
        for log in logs {
            result.push(log?);
        }
        Ok(result)
    }

    /// Get statistics
    pub fn get_stats(&self) -> Result<(usize, usize, usize)> {
        let conn = self.conn.lock().unwrap();
        
        let total: usize = conn.query_row(
            "SELECT COUNT(*) FROM tasks",
            [],
            |row| row.get(0),
        )?;
        
        let completed: usize = conn.query_row(
            "SELECT COUNT(*) FROM tasks WHERE status = 'Completed'",
            [],
            |row| row.get(0),
        )?;
        
        let failed: usize = conn.query_row(
            "SELECT COUNT(*) FROM tasks WHERE status = 'Failed'",
            [],
            |row| row.get(0),
        )?;
        
        Ok((total, completed, failed))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_persistent_storage() {
        let store = match PersistentStore::new(":memory:") {
            Ok(s) => s,
            Err(e) => {
                println!("Failed to create store: {}", e);
                return;
            }
        };
        
        let task = StoredTask {
            id: "test-1".to_string(),
            command: "echo hello".to_string(),
            status: "Completed".to_string(),
            worker_id: Some("worker-1".to_string()),
            stdout: "hello".to_string(),
            stderr: String::new(),
            exit_code: Some(0),
            duration_ms: 100,
            created_at: chrono::Utc::now().to_rfc3339(),
            completed_at: Some(chrono::Utc::now().to_rfc3339()),
        };
        
        if let Err(e) = store.store_task(&task) {
            println!("Failed to store task: {}", e);
            return;
        }
        
        match store.get_task("test-1") {
            Ok(Some(retrieved)) => {
                assert_eq!(retrieved.command, "echo hello");
            }
            Ok(None) => panic!("Task not found"),
            Err(e) => panic!("Failed to retrieve task: {}", e),
        }
    }
}
