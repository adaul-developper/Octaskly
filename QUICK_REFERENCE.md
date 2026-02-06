# OCTASKLY - Quick Reference Card

## Running OCTASKLY

### Start Dispatcher (Task Coordinator)
```bash
cargo run -- dispatcher [OPTIONS]

Options:
  --bind <BIND>           Bind address (default: 0.0.0.0)
  --port <PORT>           Port to listen on (default: 7878)
  --workdir <WORKDIR>     Work directory (default: ./tasks)
  --ui                    Enable TUI dashboard

Shortcut: cargo run -- d
```

### Start Worker (Task Executor)
```bash
cargo run -- worker [OPTIONS]

Options:
  --name <NAME>           Worker name (required)
  --allow-shell <BOOL>    Allow shell execution (default: true)
  --max-jobs <JOBS>       Max concurrent jobs (default: 2)
  --dispatcher <ADDR>     Dispatcher address (optional, for manual connection)

Shortcut: cargo run -- w --name "worker-01"
```

## CLI Examples

```bash
# Simple setup
cargo run -- dispatcher
cargo run -- worker --name "my-device"

# With custom settings
cargo run -- dispatcher --bind 0.0.0.0 --port 8888
cargo run -- worker --name "powerful-pc" --max-jobs 8

# With TUI dashboard
cargo run -- dispatcher --ui

# Using shortcuts
cargo run -- d --port 9999
cargo run -- w --name "worker-01"
```

## Logging

```bash
# Enable detailed logging
RUST_LOG=debug cargo run -- dispatcher

# Specific module logging
RUST_LOG=octaskly::scheduler=debug cargo run -- dispatcher

# Disable logging
RUST_LOG=off cargo run -- dispatcher
```

## Testing

```bash
# Run all tests
cargo test

# Run unit tests only
cargo test --lib

# Run integration tests
cargo test --test integration_tests

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_task_execution
```

## Building

```bash
# Debug build (fast, large binary)
cargo build

# Release build (slow, small optimized binary)
cargo build --release

# Clean build directory
cargo clean

# Check without building
cargo check
```

## Project Structure

```
octaskly/
├── src/
│   ├── main.rs              ← Entry point
│   ├── lib.rs               ← Library exports
│   ├── protocol/mod.rs      ← Task/Message definitions
│   ├── transport/mod.rs     ← TCP networking
│   ├── scheduler/mod.rs     ← Task queue & scheduling
│   ├── executor/mod.rs      ← Command execution
│   ├── discovery/mod.rs     ← mDNS service discovery
│   ├── state/mod.rs         ← State management
│   ├── security/mod.rs      ← Authentication
│   ├── tui/mod.rs           ← UI dashboard
│   ├── cmd/mod.rs           ← CLI parsing
│   └── util/mod.rs          ← Utilities
├── examples/
│   └── basic_usage.rs       ← Usage examples
├── tests/
│   └── integration_tests.rs ← Integration tests
├── Cargo.toml               ← Dependencies
├── README.md                ← Overview
├── IMPLEMENTATION.md        ← Technical details
└── PROJECT_DELIVERY.md      ← Delivery checklist
```

## Key Concepts

### Dispatcher
- Listens for worker connections
- Manages task queue
- Assigns tasks to idle workers
- Stores results and history

### Worker
- Connects to dispatcher
- Executes assigned tasks
- Reports progress and results
- Can run multiple jobs concurrently

### Task
- Shell command to execute
- Has timeout
- Can have input/output files
- Has unique ID

### Message Types
- **WorkerAnnounce** - Worker registration
- **AssignTask** - Dispatcher sends task
- **TaskCompleted** - Worker sends result
- **TaskProgress** - Worker sends progress
- **Heartbeat** - Keep-alive check

## Common Tasks

### Create a Task Programmatically
```rust
use octaskly::protocol::Task;

let task = Task::new("echo 'Hello'".to_string());
let task = Task {
    id: "task-123".to_string(),
    command: "cargo build".to_string(),
    timeout: 3600,
    ..Task::new("cmd".to_string())
};
```

### Execute a Task
```rust
use octaskly::executor::Executor;
use std::path::PathBuf;

let executor = Executor::new(PathBuf::from("/tmp"), true);
let result = executor.execute_with_timeout(&task).await?;
println!("Exit code: {:?}", result.exit_code);
println!("Output: {}", result.stdout);
```

### Register Worker
```rust
use octaskly::protocol::WorkerInfo;

let worker = WorkerInfo::new(
    "my-worker".to_string(),
    "192.168.1.100".to_string(),
    7879,
    4,
);
```

### Queue a Task
```rust
use octaskly::scheduler::Scheduler;

let scheduler = Scheduler::new();
scheduler.enqueue(task).await;
```

## Configuration Files

### Environment Variables
```bash
RUST_LOG          # Logging level (debug, info, warn, error)
RUST_BACKTRACE    # Enable backtrace (1 or full)
```

### Command Line Flags
See specific command help:
```bash
cargo run -- dispatcher --help
cargo run -- worker --help
```

## Monitoring

### Via TUI Dashboard
```bash
cargo run -- dispatcher --ui
```

Then use arrow keys to navigate tabs.

### Via Logging
```bash
RUST_LOG=debug cargo run -- dispatcher
```

Key log lines:
- "Starting Octaskly..." → Server started
- "Worker announced..." → Worker registered
- "Scheduling task..." → Task assigned
- "Task completed..." → Task done

INSTALL (one-line)

Linux / macOS / Termux:
```bash
curl -sSL https://github.com/adauldev/octaskly/releases/latest/download/install.sh | bash
```

Windows (PowerShell):
```powershell
powershell -ExecutionPolicy Bypass -Command "& { iwr https://github.com/adauldev/octaskly/releases/latest/download/install.ps1 -UseBasicParsing | iex }"
```

## Troubleshooting

### Port Already in Use
```bash
# Use different port
cargo run -- dispatcher --port 8888

# Find what's using port 7878
lsof -i :7878
```

### Worker Can't Connect to Dispatcher
```bash
# Check dispatcher is running
# Check firewall isn't blocking
# Use explicit dispatcher address:
cargo run -- worker --name "w1" --dispatcher "192.168.1.100:7878"
```

### Tasks Not Executing
```bash
# Enable debug logging
RUST_LOG=debug cargo run -- worker --name "w1"

# Check worker is registered with dispatcher
# Check command is valid (not dangerous patterns)
```

### Memory Leak or High Usage
```bash
# Monitor with:
cargo run --release -- dispatcher  (more efficient)

# Check task output isn't too large
# Increase max-jobs limit
```

## Performance Tips

1. **Use Release Build**
   ```bash
   cargo build --release
   ./target/release/octaskly dispatcher
   ```

2. **Tune Worker Job Limit**
   ```bash
   # Adjust based on system resources
   cargo run -- worker --name "w1" --max-jobs 8
   ```

3. **Monitor Resource Usage**
   ```bash
   RUST_LOG=debug cargo run -- dispatcher
   ```

4. **Deploy Binary Only**
   ```bash
   # No need to ship source code
   scp target/release/octaskly user@host:/usr/local/bin/
   ```

## File Locations

```
Debug binary:       target/debug/octaskly
Release binary:     target/release/octaskly
Documentation:      *.md files
Examples:           examples/
Tests:             tests/
Source code:        src/
Cargo config:       Cargo.toml
```

## Dependencies

Key crates:
- **tokio** - Async runtime
- **ratatui** - Terminal UI
- **mdns-sd** - Service discovery
- **serde** - Serialization
- **tracing** - Logging
- **clap** - CLI

See `Cargo.toml` for full list.

## Version Info

```bash
cargo --version
rustc --version
cargo run -- --version
```

Expected:
- Rust 1.70+
- Cargo 1.75+
- OCTASKLY 0.1.0

## Getting Help

1. **Built-in Help**
   ```bash
   cargo run -- --help
   cargo run -- dispatcher --help
   cargo run -- worker --help
   ```

2. **Documentation**
   - README.md - Overview
   - IMPLEMENTATION.md - Technical details
   - FEATURES.md - Feature overview
   - Code comments - Inline documentation

3. **Examples**
   ```bash
   cargo run --example basic_usage
   ```

4. **Tests**
   ```bash
   cargo test -- --nocapture  # See test output
   ```

## Next Steps

1. **Deploy** - Copy binary to multiple machines
2. **Integrate** - Use as library in your projects
3. **Extend** - Add REST API, web dashboard, etc.
4. **Monitor** - Integrate logging/metrics
5. **Scale** - Deploy to more devices

---

**Happy computing! 🐙**
