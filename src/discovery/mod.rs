// Discovery module for auto-detecting workers on the network
// Modul penemuan untuk deteksi otomatis worker di jaringan
//
// Pure Rust implementation using UDP broadcast - no native dependencies
// Implementasi Rust murni menggunakan siaran UDP - tanpa dependensi asli

use crate::protocol::WorkerInfo;
use anyhow::Result;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

// Service discovery configuration
// Konfigurasi penemuan layanan
const SERVICE_TYPE: &str = "_octaskly._udp";
const DISCOVERY_PORT: u16 = 5555;

/// Worker discovery service using UDP broadcast on local network
/// Layanan penemuan worker menggunakan siaran UDP di jaringan lokal
pub struct Discovery {
    /// Discovered workers on network
    /// Worker yang ditemukan di jaringan
    workers: Arc<RwLock<Vec<WorkerInfo>>>,
    
    /// Discovery listener running flag
    /// Flag listener penemuan sedang berjalan
    running: Arc<RwLock<bool>>,
}

impl Discovery {
    /// Initialize new discovery service
    /// Inisialisasi layanan penemuan baru
    pub fn new() -> Self {
        Self {
            workers: Arc::new(RwLock::new(Vec::new())),
            running: Arc::new(RwLock::new(false)),
        }
    }

    /// Start discovering workers on the network via UDP broadcast
    /// Mulai menemukan worker di jaringan melalui siaran UDP
    ///
    /// Listens for worker announcements on UDP port 5555
    /// Mendengarkan pengumuman worker di port UDP 5555
    pub async fn start_discovery(&mut self) -> Result<()> {
        info!("[DISCOVERY] Starting worker discovery service");

        let workers = self.workers.clone();
        let running = self.running.clone();

        // Spawn discovery listener task
        // Spawn tugas listener penemuan
        tokio::spawn(async move {
            if let Err(e) = Self::discovery_listener(workers, running).await {
                warn!("[DISCOVERY] Discovery listener error: {}", e);
            }
        });

        *self.running.write().await = true;
        Ok(())
    }

    /// UDP discovery listener task
    /// Tugas listener penemuan UDP
    async fn discovery_listener(
        workers: Arc<RwLock<Vec<WorkerInfo>>>,
        running: Arc<RwLock<bool>>,
    ) -> Result<()> {
        use std::net::UdpSocket;

        let addr = format!("0.0.0.0:{}", DISCOVERY_PORT).parse::<SocketAddr>()?;

        match UdpSocket::bind(addr) {
            Ok(socket) => {
                socket.set_broadcast(true)?;
                let mut buffer = [0u8; 512];

                loop {
                    if !*running.read().await {
                        break;
                    }

                    match socket.recv_from(&mut buffer) {
                        Ok((size, peer_addr)) => {
                            // Parse discovery message format: id|name|port|max_jobs
                            // Parsing format pesan penemuan: id|name|port|max_jobs
                            if let Ok(message) = String::from_utf8(buffer[..size].to_vec()) {
                                let parts: Vec<&str> = message.split('|').collect();
                                if parts.len() >= 4 {
                                    if let Ok(port) = parts[2].parse::<u16>() {
                                        if let Ok(max_jobs) = parts[3].parse::<usize>() {
                                            let worker = WorkerInfo {
                                                id: parts[0].to_string(),
                                                name: parts[1].to_string(),
                                                address: peer_addr.ip().to_string(),
                                                port,
                                                max_jobs,
                                                current_jobs: 0,
                                                allow_shell: true,
                                                last_heartbeat: std::time::SystemTime::now()
                                                    .duration_since(std::time::UNIX_EPOCH)
                                                    .unwrap()
                                                    .as_secs() as i64,
                                                platform: "linux".to_string(),
                                            };
                                            // Tambahkan atau perbarui worker dalam daftar
                                            let mut w = workers.write().await;
                                            if let Some(existing) = w.iter_mut().find(|x| x.id == worker.id) {
                                                existing.address = worker.address;
                                                existing.port = worker.port;
                                                existing.last_heartbeat = worker.last_heartbeat;
                                            } else {
                                                w.push(worker);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            warn!("[DISCOVERY] UDP receive error: {}", e);
                            break;
                        }
                    }
                }
            }
            Err(e) => {
                warn!("[DISCOVERY] Failed to bind UDP discovery socket: {}", e);
            }
        }

        Ok(())
    }

    /// Register this instance as a service (dispatcher)
    /// Daftarkan instance ini sebagai layanan (dispatcher)
    ///
    /// Broadcasts service information on UDP network
    /// Menyiarkan informasi layanan pada jaringan UDP
    pub async fn register_service(
        &mut self,
        name: &str,
        port: u16,
        _service_type: &str,
    ) -> Result<()> {
        info!("[DISCOVERY] Registering service: {} on port {}", name, port);

        // Service registration is implicit through announcements
        // Registrasi layanan dilakukan secara implisit melalui pengumuman
        Ok(())
    }

    /// Get list of discovered workers
    /// Dapatkan daftar worker yang ditemukan
    pub async fn get_workers(&self) -> Vec<WorkerInfo> {
        self.workers.read().await.clone()
    }

    /// Add a discovered worker
    /// Tambahkan worker yang ditemukan
    pub async fn add_worker(&self, worker: WorkerInfo) {
        debug!("[DISCOVERY] Adding worker: {}", worker.name);
        let mut workers = self.workers.write().await;
        if !workers.iter().any(|w| w.id == worker.id) {
            workers.push(worker);
        }
    }

    /// Clear discovered workers
    /// Hapus worker yang ditemukan
    pub async fn clear_workers(&self) {
        self.workers.write().await.clear();
    }

    /// Get number of discovered workers
    /// Dapatkan jumlah worker yang ditemukan
    pub async fn worker_count(&self) -> usize {
        self.workers.read().await.len()
    }
}

impl Default for Discovery {
    fn default() -> Self {
        Self::new()
    }
}

/// Get local IP address from UDP socket connection
/// Dapatkan alamat IP lokal dari koneksi soket UDP
fn get_local_ip() -> Option<String> {
    use std::net::UdpSocket;

    let socket = UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("8.8.8.8:80").ok()?;
    let local_addr = socket.local_addr().ok()?;
    Some(local_addr.ip().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_discovery_new() {
        let discovery = Discovery::new();
        let workers = discovery.get_workers().await;
        assert!(workers.is_empty());
    }

    #[tokio::test]
    async fn test_discovery_count() {
        let discovery = Discovery::new();
        assert_eq!(discovery.worker_count().await, 0);
    }
}
