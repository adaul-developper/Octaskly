use clap::{Parser, Subcommand};
use std::path::PathBuf;

// Command-line interface structure with subcommand support
// Struktur antarmuka baris perintah dengan dukungan subperintah
#[derive(Parser, Debug)]
#[command(name = "octaskly")]
#[command(about = "Offline Compute Task Coordinator - Turn your local network into a compute cluster", long_about = None)]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
}

// Available application commands
// Perintah aplikasi yang tersedia
#[derive(Subcommand, Debug)]
pub enum Command {
    // Dispatcher mode for managing task distribution
    // Mode dispatcher untuk mengelola distribusi tugas
    /// Run as dispatcher (task coordinator)
    Dispatcher {
        /// Listen address for incoming connections
        /// Alamat untuk koneksi masuk
        #[arg(long, default_value = "0.0.0.0")]
        bind: String,

        /// Network port for communication
        /// Port jaringan untuk komunikasi
        #[arg(long, default_value = "7878")]
        port: u16,

        /// Directory for storing task data
        /// Direktori untuk menyimpan data tugas
        #[arg(long, default_value = "./tasks")]
        workdir: PathBuf,

        /// Enable interactive dashboard interface
        /// Aktifkan antarmuka dasbor interaktif
        #[arg(long)]
        ui: bool,
    },
    // Worker mode for executing distributed tasks
    // Mode worker untuk menjalankan tugas terdistribusi
    /// Run as worker (task executor)
    Worker {
        /// Unique identifier for this worker
        /// Pengenal unik untuk worker ini
        #[arg(long)]
        name: String,

        /// Allow execution of shell commands
        /// Izinkan eksekusi perintah shell
        #[arg(long, default_value = "true")]
        allow_shell: bool,

        /// Number of concurrent tasks to support
        /// Jumlah tugas bersamaan yang didukung
        #[arg(long, default_value = "2")]
        max_jobs: usize,

        /// List of dispatcher addresses for manual connection
        /// Daftar alamat dispatcher untuk koneksi manual
        #[arg(long)]
        dispatcher: Option<Vec<String>>,
    },
    /// Shortcut: dispatcher
    D {
        #[arg(long, default_value = "0.0.0.0")]
        bind: String,
        #[arg(long, default_value = "7878")]
        port: u16,
        #[arg(long)]
        ui: bool,
    },
    /// Shortcut: worker
    W {
        #[arg(long)]
        name: String,
        #[arg(long, default_value = "2")]
        max_jobs: usize,
    },
}

impl Cli {
    // Parse CLI arguments and normalize command shortcuts
    // Parsing argumen CLI dan normalisasi pintasan perintah
    pub fn parse_and_run() -> Result<Command, anyhow::Error> {
        let cli = Self::parse();
        
        let cmd = match cli.command {
            Some(Command::D { bind, port, ui }) => {
                Command::Dispatcher {
                    bind,
                    port,
                    workdir: PathBuf::from("./tasks"),
                    ui,
                }
            }
            Some(Command::W { name, max_jobs }) => {
                Command::Worker {
                    name,
                    allow_shell: true,
                    max_jobs,
                    dispatcher: None,
                }
            }
            other => other.unwrap_or_else(|| {
                eprintln!("Usage: octaskly <dispatcher | worker | d | w>");
                std::process::exit(1);
            }),
        };

        Ok(cmd)
    }
}
