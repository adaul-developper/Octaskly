use anyhow::Result;
use std::path::Path;
use tracing::info;

/// Get local IP address
pub fn get_local_ip() -> Option<String> {
    let socket = std::net::UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("8.8.8.8:80").ok()?;
    let local_addr = socket.local_addr().ok()?;
    Some(local_addr.ip().to_string())
}

/// Setup tracing/logging
pub fn setup_logging() {
    use tracing_subscriber::EnvFilter;

    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .with_target(false)
        .with_thread_ids(false)
        .with_level(true)
        .init();

    info!("Logging initialized");
}

/// Create directory if not exists
pub async fn ensure_dir(path: &Path) -> Result<()> {
    if !path.exists() {
        tokio::fs::create_dir_all(path).await?;
    }
    Ok(())
}

/// Get platform info
pub fn get_platform_info() -> String {
    format!(
        "{} {}",
        std::env::consts::OS,
        std::env::consts::ARCH,
    )
}

/// Format bytes to human readable
pub fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
    let mut size = bytes as f64;
    let mut unit_idx = 0;

    while size >= 1024.0 && unit_idx < UNITS.len() - 1 {
        size /= 1024.0;
        unit_idx += 1;
    }

    format!("{:.2} {}", size, UNITS[unit_idx])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_bytes() {
        assert!(format_bytes(512).contains("B"));
        assert!(format_bytes(1024).contains("KB"));
    }
}
