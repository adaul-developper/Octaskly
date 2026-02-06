use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::process::Child;
use std::time::Duration;

/// Resource limits for task execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// Maximum CPU time in seconds
    pub cpu_limit_secs: u64,
    
    /// Maximum memory in MB
    pub memory_limit_mb: u64,
    
    /// Maximum disk space in MB
    pub disk_limit_mb: u64,
    
    /// Maximum number of open files
    pub max_open_files: u64,
    
    /// Maximum number of child processes
    pub max_processes: u64,
    
    /// Task timeout in seconds
    pub timeout_secs: u64,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            cpu_limit_secs: 3600,      // 1 hour
            memory_limit_mb: 2048,     // 2GB
            disk_limit_mb: 10240,      // 10GB
            max_open_files: 1024,
            max_processes: 100,
            timeout_secs: 3600,        // 1 hour
        }
    }
}

impl ResourceLimits {
    pub fn strict() -> Self {
        Self {
            cpu_limit_secs: 300,       // 5 minutes
            memory_limit_mb: 512,      // 512MB
            disk_limit_mb: 1024,       // 1GB
            max_open_files: 256,
            max_processes: 10,
            timeout_secs: 300,         // 5 minutes
        }
    }

    pub fn relaxed() -> Self {
        Self {
            cpu_limit_secs: 86400,     // 24 hours
            memory_limit_mb: 8192,     // 8GB
            disk_limit_mb: 102400,     // 100GB
            max_open_files: 4096,
            max_processes: 1000,
            timeout_secs: 86400,       // 24 hours
        }
    }

    /// Apply resource limits to a process
    pub fn apply_to_process(&self, _child: &mut Child) -> Result<()> {
        #[cfg(unix)]
        {
            use rlimit::Resource;
            
            // CPU limit
            Resource::CPU.set(self.cpu_limit_secs, self.cpu_limit_secs)?;
            
            // Memory limit
            let mem_bytes = self.memory_limit_mb * 1024 * 1024;
            Resource::AS.set(mem_bytes, mem_bytes)?;
            Resource::DATA.set(mem_bytes, mem_bytes)?;
            
            // Open files
            Resource::NOFILE.set(self.max_open_files, self.max_open_files)?;
            
            // Processes
            Resource::NPROC.set(self.max_processes, self.max_processes)?;
        }
        
        Ok(())
    }

    /// Validate resource limits
    pub fn validate(&self) -> Result<()> {
        if self.cpu_limit_secs == 0 {
            return Err(anyhow::anyhow!("CPU limit must be > 0"));
        }
        if self.memory_limit_mb == 0 {
            return Err(anyhow::anyhow!("Memory limit must be > 0"));
        }
        if self.timeout_secs == 0 {
            return Err(anyhow::anyhow!("Timeout must be > 0"));
        }
        Ok(())
    }

    /// Get timeout as Duration
    pub fn timeout(&self) -> Duration {
        Duration::from_secs(self.timeout_secs)
    }
}

/// Monitor process resource usage
#[cfg(unix)]
pub struct ProcessMonitor {
    pid: u32,
}

#[cfg(unix)]
impl ProcessMonitor {
    pub fn new(pid: u32) -> Self {
        Self { pid }
    }

    /// Get current memory usage in MB
    pub fn get_memory_usage(&self) -> Result<u64> {
        use procfs::process::Process;
        
        let process = Process::new(self.pid as i32)?;
        let stat = process.stat()?;
        
        // RSS is in pages, convert to MB
        let page_size = 4096; // typical page size
        let memory_mb = (stat.rss * page_size) / (1024 * 1024);
        
        Ok(memory_mb as u64)
    }

    /// Get current CPU time in seconds
    pub fn get_cpu_time(&self) -> Result<u64> {
        use procfs::process::Process;
        
        let process = Process::new(self.pid as i32)?;
        let stat = process.stat()?;
        
        // utime + stime in jiffies (typically 1/100 second)
        let jiffies = stat.utime + stat.stime;
        let seconds = jiffies / 100; // assuming 100 Hz
        
        Ok(seconds)
    }

    /// Check if process exceeds limits
    pub fn check_limits(&self, limits: &ResourceLimits) -> Result<Option<String>> {
        let mem = self.get_memory_usage()?;
        if mem > limits.memory_limit_mb {
            return Ok(Some(format!("Memory limit exceeded: {} MB > {} MB", mem, limits.memory_limit_mb)));
        }

        let cpu = self.get_cpu_time()?;
        if cpu > limits.cpu_limit_secs {
            return Ok(Some(format!("CPU limit exceeded: {} s > {} s", cpu, limits.cpu_limit_secs)));
        }

        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_limits() {
        let limits = ResourceLimits::default();
        assert!(limits.validate().is_ok());
        assert_eq!(limits.timeout_secs, 3600);
    }

    #[test]
    fn test_strict_limits() {
        let limits = ResourceLimits::strict();
        assert!(limits.validate().is_ok());
        assert!(limits.cpu_limit_secs < ResourceLimits::default().cpu_limit_secs);
    }

    #[test]
    fn test_invalid_limits() {
        let mut limits = ResourceLimits::default();
        limits.cpu_limit_secs = 0;
        assert!(limits.validate().is_err());
    }
}
