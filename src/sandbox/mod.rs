use anyhow::Result;
use std::process::{Command, Child};

/// Sandbox module for strict task isolation
pub struct Sandbox {
    isolation_level: IsolationLevel,
    work_dir: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum IsolationLevel {
    None,
    Basic,
    Strict,
    VeryStrict,
}

impl Sandbox {
    /// Create a new sandbox with specified isolation level
    pub fn new(isolation_level: IsolationLevel) -> Self {
        Self {
            isolation_level,
            work_dir: None,
        }
    }

    /// Set working directory for sandboxed process
    pub fn with_work_dir(mut self, work_dir: String) -> Self {
        self.work_dir = Some(work_dir);
        self
    }

    /// Create a sandboxed process
    pub fn execute_command(&self, program: &str, args: &[&str]) -> Result<Child> {
        let mut cmd = Command::new(program);
        
        // Set working directory if specified
        if let Some(work_dir) = &self.work_dir {
            cmd.current_dir(work_dir);
        }

        // Apply isolation based on level
        match self.isolation_level {
            IsolationLevel::None => {
                // No isolation
            }
            IsolationLevel::Basic => {
                cmd.env_clear();
                cmd.env("PATH", "/usr/bin:/bin");
                cmd.env("HOME", "/tmp");
            }
            IsolationLevel::Strict => {
                cmd.env_clear();
                cmd.env("PATH", "/usr/bin:/bin");
                cmd.env("HOME", "/tmp");
                // On Linux, could use: unshare syscall or other mechanisms
                #[cfg(unix)]
                {
                    // Similar to: unshare(CLONE_NEWNS | CLONE_NEWPID | CLONE_NEWIPC)
                    // For now, just apply basic restrictions
                    cmd.env("TMPDIR", "/tmp");
                }
            }
            IsolationLevel::VeryStrict => {
                cmd.env_clear();
                cmd.env("PATH", "");
                cmd.env("HOME", "/tmp");
                #[cfg(unix)]
                {
                    cmd.env("TMPDIR", "/tmp");
                }
            }
        }

        cmd.args(args);
        let child = cmd.spawn()?;
        Ok(child)
    }

    /// Check if a path is safe to access in sandbox
    pub fn is_path_allowed(&self, path: &str) -> bool {
        let allowed_prefixes = match self.isolation_level {
            IsolationLevel::None => vec!["/"],
            IsolationLevel::Basic => vec!["/tmp", "/home"],
            IsolationLevel::Strict => vec!["/tmp"],
            IsolationLevel::VeryStrict => vec!["/tmp"],
        };

        for prefix in allowed_prefixes {
            if path.starts_with(prefix) {
                return true;
            }
        }
        false
    }

    /// Get current isolation level
    pub fn isolation_level(&self) -> IsolationLevel {
        self.isolation_level
    }

    /// Create a temporary isolated directory
    pub async fn create_isolated_workspace(&self) -> Result<String> {
        let tempdir = tempfile::tempdir()?;
        Ok(tempdir.path().to_string_lossy().to_string())
    }
}

impl Default for Sandbox {
    fn default() -> Self {
        Self::new(IsolationLevel::Basic)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sandbox_creation() {
        let sandbox = Sandbox::new(IsolationLevel::Strict);
        assert_eq!(sandbox.isolation_level(), IsolationLevel::Strict);
    }

    #[test]
    fn test_sandbox_with_work_dir() {
        let sandbox = Sandbox::new(IsolationLevel::Basic).with_work_dir("/tmp".to_string());
        assert_eq!(sandbox.work_dir, Some("/tmp".to_string()));
    }

    #[test]
    fn test_path_allowed() {
        let sandbox = Sandbox::new(IsolationLevel::Strict);
        assert!(sandbox.is_path_allowed("/tmp/test"));
        assert!(!sandbox.is_path_allowed("/home/test"));
        assert!(!sandbox.is_path_allowed("/etc/passwd"));
    }

    #[test]
    fn test_path_allowed_basic() {
        let sandbox = Sandbox::new(IsolationLevel::Basic);
        assert!(sandbox.is_path_allowed("/tmp/test"));
        assert!(sandbox.is_path_allowed("/home/test"));
        assert!(!sandbox.is_path_allowed("/etc/passwd"));
    }

    #[test]
    fn test_sandbox_default() {
        let sandbox = Sandbox::default();
        assert_eq!(sandbox.isolation_level(), IsolationLevel::Basic);
    }
}
