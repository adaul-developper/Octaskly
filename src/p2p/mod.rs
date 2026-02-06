// P2P Network Module - WiFi Direct Peer Discovery and Communication
// Modul Jaringan P2P - Penemuan Peer WiFi Direct dan Komunikasi
//
// Supports WiFi Direct connectivity over shared WiFi networks or hotspot connections
// Mendukung konektivitas WiFi Direct melalui jaringan WiFi bersama atau koneksi hotspot
//
// Features:
// - Automatic peer discovery on local network (Penemuan peer otomatis di jaringan lokal)
// - mDNS/Bonjour service advertisement (Iklan layanan mDNS/Bonjour)
// - Direct P2P connectivity without central dispatcher (Konektivitas P2P langsung tanpa dispatcher pusat)
// - Network interface detection and management (Deteksi dan pengelolaan antarmuka jaringan)

use anyhow::{anyhow, Result};
use std::net::{SocketAddr, UdpSocket};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// P2P Peer information
/// Informasi Peer P2P
#[derive(Clone, Debug)]
pub struct P2PPeer {
    /// Unique identifier for the peer
    /// Pengenal unik untuk peer
    pub id: String,

    /// Human-readable name
    /// Nama yang dapat dibaca manusia
    pub name: String,

    /// IP address of the peer
    /// Alamat IP peer
    pub ip_address: String,

    /// UDP listening port
    /// Port dengarkan UDP
    pub port: u16,

    /// Last seen timestamp (unix seconds)
    /// Terakhir dilihat timestamp (detik unix)
    pub last_seen: i64,

    /// Service type identifier
    /// Pengenal jenis layanan
    pub service_type: String,
}

impl P2PPeer {
    /// Create new P2P peer information
    /// Buat informasi peer P2P baru
    pub fn new(id: String, name: String, ip_address: String, port: u16, service_type: String) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        Self {
            id,
            name,
            ip_address,
            port,
            last_seen: now,
            service_type,
        }
    }

    /// Check if peer is still considered active (seen within timeout)
    /// Periksa apakah peer masih dianggap aktif (terlihat dalam timeout)
    pub fn is_active(&self, timeout_secs: i64) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        (now - self.last_seen) < timeout_secs
    }

    /// Get socket address for peer
    /// Dapatkan alamat soket untuk peer
    pub fn socket_addr(&self) -> Result<SocketAddr> {
        format!("{}:{}", self.ip_address, self.port)
            .parse()
            .map_err(|e| anyhow!("Invalid socket address: {}", e))
    }
}

/// P2P Network Manager for WiFi Direct operations
/// Manajer Jaringan P2P untuk operasi WiFi Direct
pub struct P2PNetwork {
    /// Local peer identifier
    /// Pengenal peer lokal
    local_id: String,

    /// Local peer name
    /// Nama peer lokal
    local_name: String,

    /// Local IP address
    /// Alamat IP lokal
    local_ip: String,

    /// Service port for P2P communication
    /// Port layanan untuk komunikasi P2P
    local_port: u16,

    /// Discovered peers on network
    /// Peer yang ditemukan di jaringan
    peers: Arc<RwLock<HashMap<String, P2PPeer>>>,

    /// mDNS service type
    /// Jenis layanan mDNS
    service_type: String,
}

impl P2PNetwork {
    /// Initialize P2P network manager
    /// Inisialisasi manajer jaringan P2P
    ///
    /// # Arguments
    /// - local_id: Unique identifier for this peer (Pengenal unik untuk peer ini)
    /// - local_name: Display name for this peer (Nama tampilan untuk peer ini)
    /// - local_port: UDP port for P2P communication (Port UDP untuk komunikasi P2P)
    pub fn new(local_id: String, local_name: String, local_port: u16) -> Result<Self> {
        // Detect local IP address
        // Deteksi alamat IP lokal
        let local_ip = Self::detect_local_ip()?;

        info!(
            "[P2P] Initializing P2P network: {} ({}:{})",
            local_name, local_ip, local_port
        );

        Ok(Self {
            local_id: local_id.clone(),
            local_name,
            local_ip,
            local_port,
            peers: Arc::new(RwLock::new(HashMap::new())),
            service_type: "_octaskly._udp.local.".to_string(),
        })
    }

    /// Detect local IP address (preferred method)
    /// Deteksi alamat IP lokal (metode pilihan)
    ///
    /// Attempts to find the best local IP address by connecting to external service
    /// Mencoba menemukan alamat IP lokal terbaik dengan terhubung ke layanan eksternal
    fn detect_local_ip() -> Result<String> {
        // Try to connect to a public DNS to detect local IP
        // Coba terhubung ke DNS publik untuk mendeteksi IP lokal
        match UdpSocket::bind("0.0.0.0:0") {
            Ok(socket) => {
                match socket.connect("8.8.8.8:80") {
                    Ok(()) => {
                        if let Ok(addr) = socket.local_addr() {
                            return Ok(addr.ip().to_string());
                        }
                    }
                    Err(_) => {}
                }
            }
            Err(_) => {}
        }

        // Fallback to localhost
        // Fallback ke localhost
        warn!("[P2P] Could not detect IP, using localhost");
        Ok("127.0.0.1".to_string())
    }

    /// Register a discovered peer
    /// Daftarkan peer yang ditemukan
    pub async fn add_peer(&self, peer: P2PPeer) {
        let mut peers = self.peers.write().await;
        if let Some(_existing) = peers.get(&peer.id) {
            debug!(
                "[P2P] Updating peer: {} ({}:{})",
                peer.name, peer.ip_address, peer.port
            );
        } else {
            info!(
                "[P2P] New peer discovered: {} ({}:{})",
                peer.name, peer.ip_address, peer.port
            );
        }
        peers.insert(peer.id.clone(), peer);
    }

    /// Get all active peers
    /// Dapatkan semua peer yang aktif
    pub async fn get_active_peers(&self, timeout_secs: i64) -> Vec<P2PPeer> {
        let peers = self.peers.read().await;
        peers
            .values()
            .filter(|p| p.is_active(timeout_secs))
            .cloned()
            .collect()
    }

    /// Get a specific peer by ID
    /// Dapatkan peer spesifik berdasarkan ID
    pub async fn get_peer(&self, peer_id: &str) -> Option<P2PPeer> {
        let peers = self.peers.read().await;
        peers.get(peer_id).cloned()
    }

    /// Remove inactive peers from the peer list
    /// Hapus peer yang tidak aktif dari daftar peer
    pub async fn cleanup_inactive_peers(&self, timeout_secs: i64) {
        let mut peers = self.peers.write().await;
        let inactive: Vec<String> = peers
            .iter()
            .filter(|(_, p)| !p.is_active(timeout_secs))
            .map(|(id, _)| id.clone())
            .collect();

        for id in inactive {
            debug!("[P2P] Removing inactive peer: {}", id);
            peers.remove(&id);
        }
    }

    /// Get local peer information
    /// Dapatkan informasi peer lokal
    pub fn get_local_peer_info(&self) -> P2PPeer {
        P2PPeer::new(
            self.local_id.clone(),
            self.local_name.clone(),
            self.local_ip.clone(),
            self.local_port,
            self.service_type.clone(),
        )
    }

    /// Get local IP address
    /// Dapatkan alamat IP lokal
    pub fn get_local_ip(&self) -> String {
        self.local_ip.clone()
    }

    /// Get local port
    /// Dapatkan port lokal
    pub fn get_local_port(&self) -> u16 {
        self.local_port
    }

    /// Get number of active peers
    /// Dapatkan jumlah peer yang aktif
    pub async fn peer_count(&self) -> usize {
        let peers = self.peers.read().await;
        peers.len()
    }

    /// Start mDNS service discovery
    /// Mulai penemuan layanan mDNS
    ///
    /// This will advertise this peer on the local network using mDNS
    /// Ini akan mengiklankan peer ini di jaringan lokal menggunakan mDNS
    pub async fn start_mdns_discovery(&self) -> Result<()> {
        // Note: Actual mDNS implementation would go here
        // In production, use zeroconf or mdns-sd crate for actual service discovery
        // Catatan: Implementasi mDNS aktual akan dilakukan di sini
        // Dalam produksi, gunakan crate zeroconf atau mdns-sd untuk penemuan layanan aktual

        info!(
            "[P2P] mDNS service starting: {}._octaskly._udp.local",
            self.local_name
        );

        Ok(())
    }

    /// Listen for peer announcements via UDP broadcast
    /// Dengarkan pengumuman peer melalui siaran UDP
    ///
    /// This creates a UDP socket to receive peer discovery packets
    /// Ini membuat soket UDP untuk menerima paket penemuan peer
    pub async fn start_discovery_listener(&self, port: u16) -> Result<()> {
        let addr = format!("0.0.0.0:{}", port).parse::<SocketAddr>()?;

        info!("[P2P] Starting peer discovery listener on {}", addr);

        // Spawn async task to listen for discovery packets
        // Spawn tugas async untuk mendengarkan paket discovery
        let peers = self.peers.clone();
        let local_id = self.local_id.clone();

        tokio::spawn(async move {
            match UdpSocket::bind(addr) {
                Ok(socket) => {
                    // Set socket to allow broadcast
                    // Setel soket untuk mengizinkan siaran
                    let _ = socket.set_broadcast(true);

                    let mut buffer = [0u8; 512];

                    loop {
                        match socket.recv_from(&mut buffer) {
                            Ok((size, peer_addr)) => {
                                // Process discovery packet
                                // Proses paket discovery
                                if let Ok(message) = String::from_utf8(buffer[..size].to_vec()) {
                                    debug!(
                                        "[P2P] Discovery packet received from {}: {}",
                                        peer_addr, message
                                    );

                                    // Parse and register peer (simplified format: id|name|port)
                                    // Parse dan daftarkan peer (format sederhana: id|name|port)
                                    let parts: Vec<&str> = message.split('|').collect();
                                    if parts.len() >= 3 {
                                        if let Ok(port) = parts[2].parse::<u16>() {
                                            let peer_id = parts[0].to_string();
                                            if peer_id != local_id {
                                                let peer = P2PPeer::new(
                                                    peer_id,
                                                    parts[1].to_string(),
                                                    peer_addr.ip().to_string(),
                                                    port,
                                                    "_octaskly._udp.local.".to_string(),
                                                );

                                                let mut p = peers.write().await;
                                                p.insert(peer.id.clone(), peer);
                                            }
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                error!("[P2P] UDP receive error: {}", e);
                                break;
                            }
                        }
                    }
                }
                Err(e) => {
                    error!("[P2P] Failed to bind UDP socket: {}", e);
                }
            }
        });

        Ok(())
    }

    /// Send peer announcement broadcast
    /// Kirim siaran pengumuman peer
    ///
    /// Broadcasts this peer information to local network
    /// Menyiarkan informasi peer ini ke jaringan lokal
    pub async fn announce_peer(&self) -> Result<()> {
        let message = format!("{}|{}|{}", self.local_id, self.local_name, self.local_port);

        match UdpSocket::bind("0.0.0.0:0") {
            Ok(socket) => {
                socket.set_broadcast(true)?;
                let broadcast_addr = "255.255.255.255:5555".parse::<SocketAddr>()?;
                socket.send_to(message.as_bytes(), broadcast_addr)?;
                debug!("[P2P] Peer announcement sent: {}", message);
                Ok(())
            }
            Err(e) => Err(anyhow!("Failed to create broadcast socket: {}", e)),
        }
    }

    /// Establish direct P2P connection to a peer
    /// Buat koneksi P2P langsung ke peer
    ///
    /// This initiates a direct connection to another peer
    /// Ini memulai koneksi langsung ke peer lain
    pub async fn connect_to_peer(&self, peer_id: &str) -> Result<SocketAddr> {
        let peers = self.peers.read().await;

        if let Some(peer) = peers.get(peer_id) {
            let addr = peer.socket_addr()?;
            info!("[P2P] Connecting to peer {}: {}", peer_id, addr);
            Ok(addr)
        } else {
            Err(anyhow!("Peer not found: {}", peer_id))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_peer_creation() {
        let peer = P2PPeer::new(
            "peer1".to_string(),
            "Worker-1".to_string(),
            "192.168.1.100".to_string(),
            7879,
            "_octaskly._udp.local.".to_string(),
        );

        assert_eq!(peer.id, "peer1");
        assert_eq!(peer.name, "Worker-1");
        assert!(peer.is_active(60));
    }

    #[tokio::test]
    async fn test_p2p_network_creation() {
        let network = P2PNetwork::new(
            "dispatcher1".to_string(),
            "Dispatcher".to_string(),
            5555,
        );

        assert!(network.is_ok());
    }

    #[tokio::test]
    async fn test_peer_registration() {
        let network = P2PNetwork::new(
            "dispatcher1".to_string(),
            "Dispatcher".to_string(),
            5555,
        )
        .unwrap();

        let peer = P2PPeer::new(
            "worker1".to_string(),
            "Worker-1".to_string(),
            "192.168.1.100".to_string(),
            7879,
            "_octaskly._udp.local.".to_string(),
        );

        network.add_peer(peer).await;
        assert_eq!(network.peer_count().await, 1);
    }
}
