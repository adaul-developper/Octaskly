OCTASKLY v1.0.0 - Distributed Computing P2P Network

Octaskly adalah sistem distributed computing yang memungkinkan berbagi
resource komputasi antar perangkat dalam satu jaringan lokal (LAN).
Platform ini dirancang untuk memberikan solusi efficient compute sharing
dengan fokus pada simplicity, reliability, dan cross-platform support.

FEATURES

Core Capabilities
  - Automatic peer discovery (mDNS)
  - P2P network communication
  - Task queue and scheduling
  - Shell command execution
  - Real-time monitoring
  - Task persistence
  - Cross-platform support

Security Features
  - AES-256-GCM encryption
  - JWT authentication with RBAC
  - Worker whitelist validation
  - Sandbox isolation (4 levels)
  - Audit trail logging

Advanced Features
  - QUIC protocol support
  - REST API interface
  - Resource limit enforcement
  - Fault tolerance and retry
  - Performance monitoring

SYSTEM ARCHITECTURE

Transport Layer
  TCP Socket: Primary transport for P2P communication
  QUIC (Quinn): Optional fast UDP-based multiplexed protocol
  mDNS: Service discovery for automatic peer detection

Networking Stack
  Message Format: Binary (bincode) or JSON serialization
  Connection: Persistent TCP/QUIC connections
  Multiplexing: Support for concurrent message streams

Task Processing Pipeline
  
  Dispatcher                       Workers
  - Task queue                     - Task execution
  - Work distribution              - Resource monitoring
  - Worker registry                - Status reporting
  - Result aggregation             - Fault handling

Storage Architecture
  
  Database: SQLite with WAL mode
  Tables:
    - Tasks (active and historical)
    - Results (execution outcomes)
    - Audit logs (event tracking)
  
  In-Memory: Task cache for active instances
  Persistence: All results persisted to disk

Security Model
  
  Transport: AES-256-GCM encryption
  Authentication: JWT tokens with HMAC-SHA256
  Authorization: Role-based access control
    - Admin: Full system access
    - Dispatcher: Task and worker management
    - Worker: Task execution only
    - Client: Task submission only
  
  Isolation: Four-level sandbox system
    - None: No isolation
    - Basic: Environment control
    - Strict: /tmp only access
    - VeryStrict: Minimal environment

GETTING STARTED

Build

  Debug build:
    cargo build

  Release build (recommended):
    cargo build --release

  Output location:
    ./target/release/octaskly (release binary)
    ./target/debug/octaskly (debug binary)

Run Dispatcher
  
  Start dispatcher node:
    ./target/release/octaskly dispatcher --port 5555
  
  With custom configuration:
    ./target/release/octaskly dispatcher \
      --port 5555 \
      --api-port 3000 \
      --db-path ./data/octaskly.db \
      --secret-key "your-secure-key"
  
  Dispatcher responsibilities:
    - Accept worker connections
    - Manage task queue
    - Assign tasks to workers
    - Persist results to database
    - Serve REST API

Run Worker
  
  Start worker node:
    ./target/release/octaskly worker --dispatcher-addr 127.0.0.1:5555
  
  With custom configuration:
    ./target/release/octaskly worker \
      --dispatcher-addr 192.168.1.100:5555 \
      --name "worker-001" \
      --max-jobs 4
  
  Worker responsibilities:
    - Connect to dispatcher
    - Receive and execute tasks
    - Monitor resource usage
    - Report task results
    - Handle graceful shutdown

REST API

Create Task
  
  POST /api/v1/tasks
  Header: Authorization: Bearer <TOKEN>
  Body:
    {
      "command": "echo hello world",
      "timeout": 3600,
      "inputs": ["input.txt"],
      "outputs": ["output.txt"]
    }

List Tasks
  
  GET /api/v1/tasks
  Header: Authorization: Bearer <TOKEN>

Get Task Details
  
  GET /api/v1/tasks/{task-id}
  Header: Authorization: Bearer <TOKEN>

Get Statistics
  
  GET /api/v1/stats
  Header: Authorization: Bearer <TOKEN>

Health Check
  
  GET /health

Task Structure
  
  {
    "id": "unique-task-id",
    "command": "executable command",
    "status": "queued|running|completed|failed",
    "worker_id": "assigned-worker",
    "timeout": 3600,
    "inputs": ["input_file_list"],
    "outputs": ["output_file_list"],
    "created_at": "ISO8601_timestamp",
    "completed_at": "ISO8601_timestamp"
  }

TESTING & BENCHMARKING

Run Tests
  
  All tests:
    cargo test --lib
  
  Specific module:
    cargo test security_enhanced::tests
    cargo test persistence::tests
    cargo test auth::tests
  
  Show output:
    cargo test -- --nocapture
  
  Test coverage:
    Current: 35+ unit & integration tests passing (see tests/ for details)

INSTALLATION (one-line)

Automatic installer (recommended):

Linux / macOS / Termux
```bash
curl -sSL https://github.com/adauldev/octaskly/releases/latest/download/install.sh | bash
```

Windows (PowerShell)
```powershell
powershell -ExecutionPolicy Bypass -Command "& { iwr https://github.com/adauldev/octaskly/releases/latest/download/install.ps1 -UseBasicParsing | iex }"
```

Manual install (from release artifacts)

Download the appropriate tar.gz or zip from GitHub Releases and extract/move to a folder in your PATH. The CI produces tar.gz files that preserve executable permissions so no manual chmod is required.

Benchmarking
  
  Run benchmarks:
    cargo bench
  
  Profile performance:
    cargo build --release
    perf record ./target/release/octaskly dispatcher
    perf report

Logging

  Set log level:
    RUST_LOG=debug cargo run -- dispatcher
    RUST_LOG=info,octaskly=debug cargo run -- worker
  
  Log levels:
    trace: Most detailed
    debug: Development information
    info: General information
    warn: Warning messages
    error: Error messages only

CONFIGURATION

Command Line Arguments
  
  Dispatcher:
    --port PORT              Listening port (default: 5555)
    --api-port PORT          REST API port (default: 3000)
    --db-path PATH           Database path (default: ./data/octaskly.db)
    --secret-key KEY         Encryption key
    --workers COUNT          Worker thread count
  
  Worker:
    --dispatcher-addr ADDR   Dispatcher address
    --name NAME              Worker node name
    --max-jobs COUNT         Maximum concurrent jobs

Environment Variables
  
  OCTASKLY_PORT              Override listening port
  OCTASKLY_DB_PATH           Override database location
  OCTASKLY_API_PORT          Override API port
  OCTASKLY_SECRET_KEY        Override encryption key
  RUST_LOG                 Set logging level

DATABASE

Schema

  Tasks Table
    id (TEXT PRIMARY KEY)          - Task identifier
    command (TEXT NOT NULL)        - Command to execute
    status (TEXT NOT NULL)         - Current status
    worker_id (TEXT)               - Assigned worker
    stdout (TEXT)                  - Standard output
    stderr (TEXT)                  - Standard error
    exit_code (INTEGER)            - Exit code
    duration_ms (INTEGER)          - Execution time
    created_at (TEXT NOT NULL)     - Creation time
    completed_at (TEXT)            - Completion time
  
  Audit Log Table
    id (INTEGER PRIMARY KEY)       - Entry identifier
    timestamp (TEXT NOT NULL)      - Event time
    event_type (TEXT NOT NULL)     - Event category
    worker_id (TEXT)               - Associated worker
    task_id (TEXT)                 - Associated task
    details (TEXT)                 - Event details

Operations
  
  Backup database:
    cp ./data/octaskly.db ./data/octaskly.db.backup
  
  Query tasks:
    sqlite3 ./data/octaskly.db "SELECT * FROM tasks;"
  
  Export results:
    sqlite3 -header -csv ./data/octaskly.db \
      "SELECT id, status, exit_code FROM tasks;" > results.csv
