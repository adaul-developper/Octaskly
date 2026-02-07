# Octaskly - Distributed Task Orchestration

**Distributed task orchestration system with peer-to-peer resource sharing for seamless compute distribution across local networks.**

![Octaskly Logo](img/logo/octaskly.jpeg)

**Version**: 1.0.0 | **Release**: February 7, 2026 | **Status**: ✅ Production Ready

---

## What Is Octaskly?

Octaskly adalah sistem distributed computing yang memungkinkan automatic resource sharing antar perangkat dalam jaringan lokal. Dispatcher mengelola task queue, workers mengeksekusi tugas, dan peer-to-peer discovery membuat semuanya auto-connected tanpa konfigurasi manual.

Octaskly is a modern distributed computing platform enabling automatic resource sharing across local network devices with zero-configuration peer discovery and intelligent task distribution.

### Key Value
- **Zero Configuration**: Auto-discovery dengan mDNS - tidak perlu setup manual networking
- **Cross-Platform**: Windows, macOS, Linux dengan installers yang sama atau simple as double-click
- **Production Ready**: Encryption, authentication, auditing, resource limits built-in
- **Easy to Use**: CLI-based dengan restful API untuk integrasi

---

## Quick Navigation

### I'm a... (Choose Your Path)

#### 👤 **End User** (Install & Run)
- **Goal**: Install Octaskly dan mulai pakai untuk mendistribusikan tasks
- **Time**: 5-10 menit
- **Start Here**: [User Installation Guide](USER_INSTALLATION.md)
  - Windows: [USER_INSTALLATION_WINDOWS.md](USER_INSTALLATION_WINDOWS.md)
  - macOS: [USER_INSTALLATION_MACOS.md](USER_INSTALLATION_MACOS.md)
  - Linux: [USER_INSTALLATION_LINUX.md](USER_INSTALLATION_LINUX.md)
  - Android Termux: [USER_INSTALLATION_TERMUX.md](USER_INSTALLATION_TERMUX.md)

#### 👨‍💻 **developer** (Clone, Analyze, Contribute)
- **Goal**: Understand codebase, build from source, contribute features
- **Time**: 30-60 menit untuk setup
- **Start Here**: 
  1. Clone: `git clone https://github.com/adauldev/octaskly.git`
  2. Read: [IMPLEMENTATION.md](IMPLEMENTATION.md) - Architecture & design
  3. Read: [PROJECT_STRUCTURE.md](PROJECT_STRUCTURE.md) - Code organization
  4. Build: `cargo build --release`
  5. Explore code in `src/` directory

#### 🚀 **Operator** (Deploy & Manage)
- **Goal**: Setup dalam production environment, manage security, monitor performance
- **Time**: 1-2 jam untuk production setup
- **Start Here**:
  1. [Installation](USER_INSTALLATION.md) - Get binary
  2. [DEPLOYMENT.md](DEPLOYMENT.md) - Production setup guide
  3. [SECURITY.md](SECURITY.md) - Security configuration & best practices
  4. [HELP.md](HELP.md) - Command reference & troubleshooting

---

## Core Features

### 🎯 Automatic Peer Discovery
- **mDNS Protocol**: Zero-configuration networking
- **UDP Broadcasts**: Workers announce availability automatically
- **Real-time**: Dispatcher discovers peers instantly
- **No Manual Setup**: Just start nodes on same network

### 🧠 Intelligent Task Distribution
- **Resource-Aware**: Matches tasks to worker capabilities (CPU, memory, GPU)
- **Load Balancing**: Distributes tasks across available workers
- **P2P Direct**: Can send tasks directly worker-to-worker when optimal
- **Auto-Failover**: Falls back to dispatcher-coordinated dispatch if needed

### 🔄 Hybrid Architecture
```
Centralized Dispatcher   ↔  P2P Network Layer  ↔  Workers
(Task coordination)          (Direct comm)         (Execution)
```
- TCP for reliable messaging
- UDP for discovery
- Works locally or on LAN - no internet required

### 🛡️ Enterprise Security
- **Encryption**: AES-256-GCM for all communications
- **Authentication**: JWT tokens + HMAC-SHA256
- **Authorization**: Role-based access control (Admin, Dispatcher, Worker, Client)
- **Sandboxing**: 4-level execution isolation (None, Basic, Strict, VeryStrict)
- **Auditing**: Complete audit logs of all actions
- **Resource Limits**: CPU time, memory, disk, file descriptors per task

### 📊 Advanced Capabilities
- **Persistent Storage**: SQLite-backed task queue
- **Performance Metrics**: Real-time monitoring & stats
- **REST API**: Programmatic task submission
- **QUIC Protocol**: High-performance UDP transport option
- **Automatic Retry**: Configurable backoff for failed tasks

---

## Installation (5 Minute Setup)

### Option 1: Download Installer (Easiest) ⭐

**Perfect for**: End users who want to install and go

Choose your OS:
- **🪟 Windows**: Download `.exe` → Double-click
- **🍎 macOS**: Download `.pkg` → Double-click  
- **🐧 Linux**: Use package manager (`.deb`, `.rpm`, `.run`)
- **🤖 Android**: Install Termux app → Clone & build

📖 **[Detailed Installation Guide →](USER_INSTALLATION.md)**

### Option 2: One-Line Script Install

```bash
# Linux, macOS, WSL, Termux
bash scripts/install.sh install
```

### Option 3: Build from Source

```bash
# Clone repository
git clone https://github.com/adauldev/octaskly.git
cd octaskly

# Build release binary
cargo build --release

# Binary location: ./target/release/octaskly (or .exe on Windows)
```

### Option 4: Verification

After any installation method:
```bash
# Verify it worked
octaskly --version
# Output: octaskly 1.0.0 ✓
```

---

## Quick Start (3 Minutes)

### Terminal 1: Start Dispatcher

```bash
octaskly dispatcher --port 7878
```

Output will show:
```
2026-02-07 14:30:45  Dispatcher listening on 0.0.0.0:7878
2026-02-07 14:30:45  P2P discovery enabled on 0.0.0.0:5555
2026-02-07 14:30:45  Ready to accept workers...
```

### Terminal 2: Start Worker(s)

```bash
# Worker 1
octaskly worker --name worker-01

# Worker 2 (in another terminal)
octaskly worker --name worker-02
```

Workers auto-discover dispatcher and register.

### Terminal 3: Submit a Task

```bash
# Submit task via API
curl -X POST http://localhost:3000/api/v1/tasks \
  -H "Content-Type: application/json" \
  -d '{
    "command": "echo Octaskly rocks!",
    "timeout": 60
  }'

# Or via CLI (if implemented)
octaskly submit "echo Octaskly rocks!"
```

**Done!** Your task is distributed and executed.

---

## Complete Command Reference

### Global Options

```bash
octaskly --version                 # Show version
octaskly --help                    # Show all commands
octaskly <command> --help          # Show command help
octaskly <command> --verbose       # Enable debug logging
```

### Dispatcher Command

Start the central task orchestrator and coordinator:

```bash
octaskly dispatcher [OPTIONS]
```

**All Options**:
```
NETWORKING:
  --port PORT                      
    Port to listen on (TCP for API/workers)
    Default: 7878
    Example: octaskly dispatcher --port 8080

  --bind ADDRESS                   
    Network interface to bind to
    Default: 0.0.0.0 (listen on all interfaces)
    Example: octaskly dispatcher --bind 192.168.1.100

WORKER MANAGEMENT:
  --max-workers COUNT              
    Maximum concurrent workers that can connect
    Default: 10
    Example: octaskly dispatcher --max-workers 20

  --discovery-port PORT            
    UDP port for peer discovery (mDNS/broadcasts)
    Default: 5555
    Example: octaskly dispatcher --discovery-port 5555

TASK SETTINGS:
  --task-timeout SECONDS           
    Default timeout for tasks (in seconds)
    Default: 3600 (1 hour)
    Example: octaskly dispatcher --task-timeout 600

  --max-queue-size SIZE            
    Maximum tasks in queue
    Default: 10000
    Example: octaskly dispatcher --max-queue-size 5000

P2P NETWORK:
  --p2p-enabled BOOL               
    Enable peer-to-peer direct task distribution
    Default: true
    Example: octaskly dispatcher --p2p-enabled false

SECURITY:
  --require-auth BOOL              
    Require authentication tokens
    Default: false
    Example: octaskly dispatcher --require-auth true

  --secret KEY                     
    Secret key for token signing
    Default: generated randomly
    Example: octaskly dispatcher --secret "$(openssl rand -hex 32)"

LOGGING:
  --verbose                        
    Enable debug logging output
    Example: octaskly dispatcher --verbose

  --log-file PATH                  
    Write logs to file instead of stderr
    Example: octaskly dispatcher --log-file /var/log/octaskly.log
```

**Common Examples**:
```bash
# Standard setup (recommended for testing)
octaskly dispatcher

# Production setup with authentication
octaskly dispatcher --port 7878 --bind 0.0.0.0 \
  --require-auth true --secret "$(openssl rand -hex 32)" \
  --verbose

# Centralized only (disable P2P)
octaskly dispatcher --p2p-enabled false

# Custom queue size for high-throughput
octaskly dispatcher --max-queue-size 50000 --max-workers 100
```

---

### Worker Command

Start a task execution node:

```bash
octaskly worker [OPTIONS]
```

**All Options**:
```
IDENTIFICATION:
  --name NAME                      
    Worker identifier (must be unique in network)
    Default: auto-generated (worker-XXXXX)
    Example: octaskly worker --name gpu-compute-01

  --id ID                          
    Internal worker ID (usually auto-assigned)
    Default: generated UUID
    Example: octaskly worker --id w-12345

DISPATCHER CONNECTION:
  --dispatcher ADDR                
    Dispatcher hostname/IP to connect to
    Default: localhost
    Example: octaskly worker --dispatcher 192.168.1.100

  --dispatcher-port PORT           
    Dispatcher port (TCP)
    Default: 7878
    Example: octaskly worker --dispatcher-port 8080

RESOURCE DECLARATION:
  --cpu-cores COUNT                
    Number of CPU cores available
    Default: auto-detect from system
    Example: octaskly worker --cpu-cores 16

  --memory MB                      
    RAM available in megabytes
    Default: auto-detect from system
    Example: octaskly worker --memory 65536

  --gpu BOOL                       
    Whether GPU is available
    Default: false
    Example: octaskly worker --gpu true

  --max-jobs COUNT                 
    Maximum concurrent jobs this worker handles
    Default: 4
    Example: octaskly worker --max-jobs 8

SECURITY:
  --sandbox-level LEVEL            
    Execution isolation level: none | basic | strict | very-strict
    Default: basic
    Example: octaskly worker --sandbox-level strict

LOGGING:
  --verbose                        
    Enable debug logging
    Example: octaskly worker --verbose

  --log-file PATH                  
    Write logs to file
    Example: octaskly worker --log-file ~/.octaskly/worker.log
```

**Common Examples**:
```bash
# Basic worker (uses auto-detected resources)
octaskly worker --name worker-01

# Worker with specific resources
octaskly worker --name compute-01 --cpu-cores 16 --memory 65536 --max-jobs 8

# GPU-enabled worker
octaskly worker --name gpu-01 --gpu true --max-jobs 4

# Connect to remote dispatcher
octaskly worker --name remote-worker \
  --dispatcher 192.168.1.100 --dispatcher-port 8080

# Production worker with strict sandbox
octaskly worker --name prod-worker-01 \
  --dispatcher dispatcher.company.com \
  --sandbox-level strict --max-jobs 8 --verbose

# Development worker on same machine
octaskly worker --name dev-worker --dispatcher localhost --port 7878
```

---

### Task Submission (Via API)

**Submit Task**:
```bash
curl -X POST http://localhost:3000/api/v1/tasks \
  -H "Content-Type: application/json" \
  -d '{
    "command": "bash -c \"echo hello && sleep 5\"",
    "timeout": 60,
    "memory_limit_mb": 512,
    "disk_limit_mb": 1000,
    "sandbox_level": "strict",
    "retry_count": 3,
    "priority": "high"
  }' \
  -H "Authorization: Bearer $TOKEN"
```

**Get Task Status**:
```bash
curl http://localhost:3000/api/v1/tasks/{task-id} \
  -H "Authorization: Bearer $TOKEN"
```

**List All Tasks**:
```bash
curl http://localhost:3000/api/v1/tasks \
  -H "Authorization: Bearer $TOKEN"
```

**Cancel Task**:
```bash
curl -X DELETE http://localhost:3000/api/v1/tasks/{task-id} \
  -H "Authorization: Bearer $TOKEN"
```

📖 Full API Reference: [API_REFERENCE.md](API_REFERENCE.md)

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────┐
│          Local Network (LAN)                            │
├─────────────────────────────────────────────────────────┤
│                                                         │
│  ┌──────────────────┐                                 │
│  │   Dispatcher     │  TCP 7878 (Tasks, API)         │
│  │ (Task Scheduler) │  UDP 5555 (Discovery)          │
│  └────────┬─────────┘                                 │
│           │                                            │
│           ├──────TCP──────────┬──────────┬─────────   │
│           │                   │          │             │
│      ┌────▼──────┐   ┌────────▼──┐ ┌────▼──────┐     │
│      │ Worker-01 │   │ Worker-02 │ │ Worker-N  │     │
│      │ (Execute) │   │ (Execute) │ │ (Execute) │     │
│      └───────────┘   └───────────┘ └───────────┘     │
│           │   P2P UDP Direct Comm     │                │
│           └────────────────────────────┘               │
└─────────────────────────────────────────────────────────┘

Communication:
· TCP: Persistent connections for reliability
· UDP: Multicast for peer discovery (5555)
```

### Components

| Component | Purpose | File Location |
|-----------|---------|---------------|
| **Dispatcher** | Central task orchestration and queue management | `src/executor/` |
| **Worker** | Task execution engine with resource limits | `src/executor/` |
| **P2P Network** | Peer discovery and direct worker communication | `src/p2p/`  |
| **Protocol** | Message definitions and serialization | `src/protocol/` |
| **Discovery** | mDNS-based peer service discovery | `src/discovery/` |
| **Security** | Encryption, auth, RBAC, sandboxing | `src/security/`, `src/security_enhanced/` |
| **Persistence** | Task queue and audit log storage | `src/persistence/` |
| **API** | REST HTTP endpoints | `src/api/` |
| **Scheduler** | Task scheduling algorithms | `src/scheduler/` |

---

## Network Configuration

### Single Machine (Testing)

Workers auto-connect to `localhost`:

```bash
# Terminal 1: Start dispatcher
octaskly dispatcher --port 7878

# Terminal 2: Worker 1
octaskly worker --name test-worker-01

# Terminal 3: Worker 2
octaskly worker --name test-worker-02

# They auto-discover via localhost
```

### Local Network (Multi-Machine)

Specify dispatcher IP when connecting:

```bash
# Machine A (192.168.1.10) - Dispatcher
octaskly dispatcher --bind 192.168.1.10 --port 7878

# Machine B - Worker
octaskly worker --name worker-b \
  --dispatcher 192.168.1.10 --dispatcher-port 7878

# Machine C - Worker
octaskly worker --name worker-c \
  --dispatcher 192.168.1.10 --dispatcher-port 7878
```

### P2P Auto-Discovery

Once workers connect, they also discover each other via UDP:

```bash
# All nodes on same subnet can communicate directly P2P
# Firewall: Must allow UDP port 5555 between machines
ufw allow from 192.168.1.0/24 to any port 5555 proto udp
```

---

## Documentation by Topic

### For Installation & Setup
- [USER_INSTALLATION.md](USER_INSTALLATION.md) - **Start here**
  - [USER_INSTALLATION_WINDOWS.md](USER_INSTALLATION_WINDOWS.md)
  - [USER_INSTALLATION_MACOS.md](USER_INSTALLATION_MACOS.md)
  - [USER_INSTALLATION_LINUX.md](USER_INSTALLATION_LINUX.md)
  - [USER_INSTALLATION_TERMUX.md](USER_INSTALLATION_TERMUX.md)

### For Usage & Operations
- [HELP.md](HELP.md) - Command reference and troubleshooting
- [DEPLOYMENT.md](DEPLOYMENT.md) - Production deployment guide
- [BUILD_AND_DISTRIBUTE.md](BUILD_AND_DISTRIBUTE.md) - Building installers

### For Development & Architecture
- [IMPLEMENTATION.md](IMPLEMENTATION.md) - System architecture & design
- [PROJECT_STRUCTURE.md](PROJECT_STRUCTURE.md) - Codebase organization
- [API_REFERENCE.md](API_REFERENCE.md) - REST API specification

### For Security
- [SECURITY.md](SECURITY.md) - Security model, encryption, RBAC, sandboxing
- [NATIVE_INSTALLERS.md](NATIVE_INSTALLERS.md) - Installer security

### For Releases & Changes
- [CHANGELOG.md](CHANGELOG.md) - Version history
- [RELEASE_NOTES.md](RELEASE_NOTES.md) - Current release notes
- [INSTALLATION_REFERENCE.md](INSTALLATION_REFERENCE.md) - Installation methods index

---

## Technology Stack

### Core Runtime
- **Language**: Rust 1.70+ (systems programming)
- **Async Runtime**: Tokio (non-blocking I/O)
- **Platforms**: Windows 10+, macOS 10.13+, Linux (all distros)
- **Architectures**: x86_64, ARM64 (including Apple Silicon)

### Networking
- **Protocol**: TCP (reliable) + UDP (discovery)
- **Discovery**: mDNS (zero-config networking)
- **Serialization**: JSON (human-readable) + Bincode (compact binary)
- **Advanced**: QUIC protocol support (optional, high-performance)

### Security
- **Encryption**: AES-256-GCM (authenticated encryption)
- **Authentication**: JWT with HMAC-SHA256
- **Authorization**: Role-based access control (RBAC)
- **Storage**: SQLite with WAL (persistent task queue)

### Build & Distribution
- **Build**: Cargo and Rust compiler
- **Installers**: Platform-specific (Inno Setup, macOS PKG, DEB/RPM)
- **Scripting**: Bash (Unix), PowerShell (Windows)
- **CI/CD**: GitHub Actions

---

## Performance Benchmarks

| Scenario | Throughput | Latency | Notes |
|----------|-----------|---------|-------|
| **Single machine** (4 workers) | 100-200 tasks/sec | <50ms | Local IPC |
| **LAN** (10 workers) | 50-100 tasks/sec | 100-200ms | Network overhead |
| **Large tasks** (>1 min) | Task-dependent | N/A | CPU/memory bound |

### Scaling Guidelines

| Scale | Recommended Setup |
|-------|-------------------|
| **Development/Test** | 1 dispatcher + 1-2 workers (same machine) |
| **Small Team** | 1 dispatcher + 4-8 workers (across LAN) |
| **Production** | Dedicated hardware, optimized networking |
| **High-Throughput** | Multiple dispatchers with load balancing |

---

## Building from Source (Developer)

### Prerequisites

```bash
# Rust toolchain (includes Cargo)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Git
git --version  # Should output git version
```

### Build Commands

```bash
# Clone repo
git clone https://github.com/adauldev/octaskly.git
cd octaskly

# Debug build (faster compile, slower runtime)
cargo build

# Release build (slower compile, optimized runtime)
cargo build --release
# Binary: ./target/release/octaskly (or .exe on Windows)

# Development setup
cargo install --path .

# Run tests
cargo test

# Code quality checks
cargo fmt                   # Format
cargo clippy               # Lint
cargo check                # Syntax check
```

### Code Organization

```
src/
├── main.rs                  # CLI entry point
├── lib.rs                   # Library exports
├── api/                     # HTTP REST API handlers
├── auth/                    # JWT tokens & authentication
├── cmd/                     # CLI command structure
├── discovery/               # mDNS peer discovery
├── executor/                # Dispatcher & worker engines
├── p2p/                     # P2P networking layer
├── p2p_distribution.rs      # Task distribution algorithm
├── persistence/             # SQLite database operations
├── protocol/                # Message definitions
├── resources/               # Resource management
├── sandbox/                 # Process sandboxing & isolation
├── scheduler/               # Task scheduling
├── security/                # Encryption & auth logic
├── state/                   # Application state management  
├── transport/               # Network transport layer
├── tui/                     # Terminal UI (optional)
└── util/                    # Utility functions
```

### Contributing

1. Fork repository
2. Create feature branch: `git checkout -b feature/your-feature`
3. Read [IMPLEMENTATION.md](IMPLEMENTATION.md) for architecture
4. Make changes and test: `cargo test`
5. Format & lint: `cargo fmt && cargo clippy`
6. Submit pull request

---

## Troubleshooting

### Installation Issues

**Command not found after installation**:
```bash
# WSL/Linux: Reload shell
source ~/.bashrc
# or
source ~/.zshrc

# macOS: Reload shell or restart Terminal
source ~/.bash_profile

# Windows: Restart PowerShell/CMD
exit
# Then reopen
```

**Permission denied**:
```bash
# Make script executable
chmod +x scripts/install.sh

# Run with sudo
sudo bash scripts/install.sh install
```

### Connection Issues

**Worker can't reach dispatcher**:
```bash
# Check dispatcher is running
octaskly dispatcher --verbose

# Test connectivity from worker machine
ping dispatcher-ip-address

# Verify port is open
nc -zv dispatcher-ip 7878  # Linux/macOS
Test-NetConnection dispatcher-ip -Port 7878  # Windows
```

**P2P peers not auto-discovering**:
```bash
# Verify UDP port 5555 is accessible
netstat -an | grep 5555

# Check firewall allows UDP 5555
ufw allow 5555/udp

# Enable verbose logging
octaskly dispatcher --verbose
```

📖 **Full Troubleshooting Guide**: [HELP.md](HELP.md)

---

## FAQ

**Q: Can I run dispatcher and worker on the same machine?**  
A: Yes. Start dispatcher first, then workers in separate terminals.

**Q: How many workers can one dispatcher manage?**  
A: Default 10 (configurable with `--max-workers`). Scales to 100+ with proper setup.

**Q: What if all workers are busy?**  
A: Tasks queue up and run when resources become available.

**Q: Can different workers have different capabilities?**  
A: Yes. Declare CPU, memory, GPU for each worker with command flags. Tasks are matched to compatible workers.

**Q: Does it work over the internet (WAN)?**  
A: Currently LAN/local network only. For WAN, use explicit dispatcher IP instead of discovery.

**Q: How do I monitor tasks and worker status?**  
A: Via REST API (`curl http://localhost:3000/api/v1/...`) or CLI commands. See [API_REFERENCE.md](API_REFERENCE.md).

**Q: How do I uninstall?**  
A: Remove binary from PATH location. Check platform guide: [USER_INSTALLATION.md](USER_INSTALLATION.md)

More FAQs: [HELP.md](HELP.md)

---

## License

📄 **MIT License** - See [LICENSE](LICENSE) file for terms

---

## Getting Help

- **Installation Issues**: [USER_INSTALLATION.md](USER_INSTALLATION.md)
- **Command Reference**: [HELP.md](HELP.md)
- **Architecture Questions**: [IMPLEMENTATION.md](IMPLEMENTATION.md)
- **Deployment Help**: [DEPLOYMENT.md](DEPLOYMENT.md)
- **Security**: [SECURITY.md](SECURITY.md)
- **All Docs**: [INSTALLATION_REFERENCE.md](INSTALLATION_REFERENCE.md)

---

## Release Status

**Version**: 1.0.0  
**Last Updated**: February 7, 2026  
**Status**: ✅ Production Ready - Active Development

### ✅ Completed
- P2P peer discovery (mDNS)
- Task distribution algorithm
- Cross-platform installers
- Comprehensive documentation
- Security (encryption, auth, RBAC, sandboxing)
- Audit logging
- Resource limits

### 📋 Planned
- Homebrew, APT, RPM package support
- GUI dashboard
- Automatic updates
- Web-based monitoring
- Master-master replication

---

**Built with ❤️ by adaul-developper team.**

## Contributors

- **Lead Developer**: [@0xddekaee](https://github.com/0xddekaee)
- **Publisher**: [@adauldev](https://github.com/adauldev)

See [CONTRIBUTOR.md](CONTRIBUTOR.md) for full contributor information.
