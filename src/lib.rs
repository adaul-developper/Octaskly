// Octaskly - Offline Compute Task Coordinator
// P2P network resource sharing for compute tasks
// Koordinator Tugas Komputasi Offline - Berbagi sumber daya jaringan peer-to-peer untuk tugas komputasi

pub mod cmd;
pub mod discovery;
pub mod executor;
pub mod protocol;
pub mod scheduler;
pub mod security;
pub mod state;
pub mod transport;
pub mod tui;
pub mod util;

// P2P WiFi Direct networking module
// Modul jaringan WiFi Direct P2P
pub mod p2p;

// Enhanced features for version 2.0
// Fitur yang ditingkatkan untuk versi 2.0
pub mod security_enhanced;
pub mod persistence;
pub mod auth;
pub mod resources;
pub mod api;
pub mod transport_quic;
pub mod sandbox;

pub use cmd::Command;
pub use discovery::Discovery;
pub use executor::Executor;
pub use protocol::{Task, TaskResult, WorkerInfo};
pub use scheduler::Scheduler;
pub use state::{DispatcherState, WorkerState};
pub use transport::Transport;
pub use p2p::{P2PNetwork, P2PPeer};

// New exports
pub use security_enhanced::SecurityManager;
pub use persistence::PersistentStore;
pub use auth::AuthManager;
pub use resources::ResourceLimits;
pub use api::ApiState;
pub use transport_quic::{QuicTransport, QuicConfig};
pub use sandbox::{Sandbox, IsolationLevel};

