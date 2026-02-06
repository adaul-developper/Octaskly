DEPLOYMENT GUIDE

Pre-deployment Checklist

System Requirements
  - Rust 1.70 or later
  - 4GB RAM minimum (8GB recommended)
  - 100MB disk space for binary
  - 1GB disk space for database
  - Linux/macOS/Windows compatible

Network Requirements
  - Port 5555 (TCP/QUIC) for P2P communication
  - Port 3000 (TCP) for REST API
  - Local network connectivity
  - Firewall configured to allow ports

PRODUCTION BUILD

Compile Release Binary

  cargo build --release

  This produces optimized binary at:
    ./target/release/octaskly

  Binary size: ~50MB (stripped: ~15MB)
  Startup time: <1 second
  Memory overhead: ~50MB

Strip Debug Symbols

  For smaller deployment:
    strip ./target/release/octaskly
  
  Result: ~15MB binary size

Create Installation Package

  mkdir -p ~/octaskly/{bin,data,config,logs}
  cp ./target/release/octaskly ~/octaskly/bin/
  cp ./Cargo.toml ~/octaskly/
  cp README.md ~/octaskly/
  cp SECURITY.md ~/octaskly/

DEPLOYMENT MODELS

Model 1: Single Dispatcher + Workers

  Architecture:
    - One dispatcher node (central)
    - Multiple worker nodes (compute)
    - All on same local network
  
  Setup:
    1. Start dispatcher on powerful machine
    2. Start workers on helper devices
    3. Workers auto-discover dispatcher
    4. Submit tasks via API or CLI

  Suitable For:
    - Small teams (2-10 nodes)
    - Simple workloads
    - Low availability requirements

Model 2: High Availability

  Architecture:
    - Primary + standby dispatcher
    - Database replication
    - Worker pool with load balancing
  
  Setup:
    1. Configure database replication
    2. Set up health monitoring
    3. Configure failover mechanism
    4. Load balance API requests
  
  Suitable For:
    - Production environments
    - Critical workloads
    - Medium-large clusters

Model 3: Federated Cluster

  Architecture:
    - Multiple independent dispatchers
    - Shared database backend
    - Unified API gateway
    - Worker pool across regions
  
  Setup:
    1. Central database (PostgreSQL)
    2. API aggregation layer
    3. Worker assignment strategy
    4. Cross-region networking
  
  Suitable For:
    - Large distributed systems
    - Multi-location deployments
    - Enterprise scale

SYSTEM CONFIGURATION

Configuration Methods

  1. Command-line Arguments
     octaskly dispatcher \
       --port 5555 \
       --api-port 3000 \
       --db-path /var/octaskly/data.db \
       --secret-key "$(cat /etc/octaskly/secret)" \

  2. Environment Variables
     export OCTASKLY_PORT=5555
     export OCTASKLY_API_PORT=3000
     export OCTASKLY_DB_PATH=/var/octaskly/data.db
     export OCTASKLY_SECRET_KEY="$(cat /etc/octaskly/secret)"
     octaskly dispatcher

  3. Configuration File (future)
     Will support YAML/TOML format

Key Configuration Parameters

  Network
    - bind_address: Interface to listen on
    - port: P2P communication port
    - api_port: REST API port
    - quic_enabled: Enable QUIC transport
  
  Security
    - secret_key: Encryption/auth key
    - whitelist: Allowed worker IDs
    - require_auth: Force authentication
    - audit_logging: Enable audit logs
  
  Storage
    - db_path: SQLite database location
    - db_backup_frequency: Backup schedule
    - log_retention_days: Log retention
  
  Performance
    - worker_threads: Thread count
    - task_queue_size: Max queued tasks
    - connection_timeout: Socket timeout
    - keepalive_interval: Heartbeat frequency
  
  Resource Limits
    - default_cpu_seconds: Default CPU limit
    - default_memory_mb: Default RAM limit
    - default_timeout: Default timeout

SYSTEMD SERVICE

Create Service File

  Create: /etc/systemd/system/octaskly.service
  
  Content:
  
    [Unit]
    Description=Octaskly Distributed Computing Platform
    After=network.target
    
    [Service]
    Type=simple
    User=octaskly
    Group=octaskly
    WorkingDirectory=/opt/octaskly
    
    ExecStart=/opt/octaskly/bin/octaskly dispatcher \
      --port 5555 \
      --api-port 3000 \
      --db-path /var/lib/octaskly/data.db
    
    Environment="RUST_LOG=info"
    Environment="OCTASKLY_SECRET_KEY=%i"
    
    Restart=on-failure
    RestartSec=10
    StandardOutput=journal
    StandardError=journal
    
    [Install]
    WantedBy=multi-user.target

Enable and Start Service

  Enable on boot:
    sudo systemctl enable octaskly.service
  
  Start service:
    sudo systemctl start octaskly.service
  
  Check status:
    sudo systemctl status octaskly.service
  
  View logs:
    sudo journalctl -u octaskly.service -f

DOCKER DEPLOYMENT

Dockerfile Example

  FROM rust:1.75 as builder
  WORKDIR /app
  COPY . .
  RUN cargo build --release
  
  FROM debian:bookworm-slim
  RUN apt-get update && apt-get install -y ca-certificates
  COPY --from=builder /app/target/release/octaskly /usr/local/bin/
  
  EXPOSE 5555 3000
  ENTRYPOINT ["octaskly"]

Release artifacts and installers

For production deployments, prefer using CI-produced release archives from GitHub Releases. The `DISTRIBUTION.md` guide explains how to download verified tarballs (which preserve executable permissions) or use the automated installer scripts for each platform.

Build Docker Image

  docker build -t octaskly:1.0.0 .

Run Docker Container

  Dispatcher:
    docker run -d \
      --name octaskly-dispatcher \
      -p 5555:5555 \
      -p 3000:3000 \
      -v octaskly-data:/data \
      -e RUST_LOG=info \
      octaskly:1.0.0 dispatcher \
      --port 5555 \
      --api-port 3000 \
      --db-path /data/octask.db
  
  Worker:
    docker run -d \
      --name octaskly-worker \
      -e RUST_LOG=info \
      octaskly:1.0.0 worker \
      --dispatcher-addr dispatcher:5555 \
      --name "docker-worker"

Docker Compose

  version: '3.8'
  
  services:
    dispatcher:
      image: octaskly:1.0.0
      ports:
        - "5555:5555"
        - "3000:3000"
      environment:
        RUST_LOG: info
      volumes:
        - octaskly-data:/data
      command: >
        dispatcher
        --port 5555
        --api-port 3000
        --db-path /data/octaskly.db
    
    worker:
      image: octaskly:1.0.0
      environment:
        RUST_LOG: info
      depends_on:
        - dispatcher
      command: >
        worker
        --dispatcher-addr dispatcher:5555
        --name containerized-worker
  
  volumes:
    octaskly-data:

Start Stack

  docker-compose up -d

KUBERNETES DEPLOYMENT

Deployment Manifest

  apiVersion: apps/v1
  kind: Deployment
  metadata:
    name: octask-dispatcher
  spec:
    replicas: 1
    selector:
      matchLabels:
        app: octask-dispatcher
    template:
      metadata:
        labels:
          app: octaskly-dispatcher
      spec:
        containers:
        - name: octaskly
          image: octaskly:1.0.0
          ports:
          - containerPort: 5555
          - containerPort: 3000
          env:
          - name: RUST_LOG
            value: "info"
          - name: OCTASKLY_PORT
            value: "5555"
          - name: OCTASKLY_API_PORT
            value: "3000"
          - name: OCTASKLY_DB_PATH
            value: "/data/octaskly.db"
          volumeMounts:
          - name: data
            mountPath: /data
          resources:
            limits:
              memory: "1Gi"
              cpu: "1000m"
            requests:
              memory: "512Mi"
              cpu: "500m"
        volumes:
        - name: data
          persistentVolumeClaim:
            claimName: octaskly-data

Service Manifest

  apiVersion: v1
  kind: Service
  metadata:
    name: octask-dispatcher
  spec:
    type: LoadBalancer
    selector:
      app: octask-dispatcher
    ports:
    - name: p2p
      port: 5555
      targetPort: 5555
      protocol: TCP
    - name: api
      port: 3000
      targetPort: 3000
      protocol: TCP

MONITORING AND LOGGING

Logging Configuration

  Set Log Level:
    RUST_LOG=debug cargo run -- dispatcher
    RUST_LOG=trace,octaskly=debug cargo run -- worker
  
  Log Levels:
    trace  -most detailed
    debug  - development info
    info   - general events
    warn   - warnings
    error  - errors only
  
  Log Output:
    Default: stdout/stderr
    File: Redirect with systemd/docker
    Structured: JSON with tracing-subscriber

Health Monitoring

  Check Health Endpoint:
    curl http://localhost:3000/health
  
  Expected Response:
    {
      "status": "healthy",
      "version": "0.2.0",
      "uptime_seconds": 3600
    }
  
  Set up Monitoring:
    - HTTP endpoint check every 30s
    - Alert if response > 1s
    - Alert if status != healthy

Metrics Collection (future)

  Will support:
    - Prometheus metrics
    - Custom performance metrics
    - Resource usage tracking
    - Request latency histograms

DATABASE BACKUP

SQLite Backup

  Automated backup (systemd timer):
    [Unit]
    Description=Octaskly Database Backup
    
    [Timer]
    OnCalendar=daily
    OnBootSec=10min
    
    [Install]
    WantedBy=timers.target
  
  Backup Script:
    #!/bin/bash
    BACKUP_DIR="/var/backups/octaskly"
    DB_PATH="/var/lib/octaskly/data.db"
    TIMESTAMP=$(date +%Y%m%d_%H%M%S)
    
    mkdir -p $BACKUP_DIR
    cp $DB_PATH $BACKUP_DIR/octaskly_$TIMESTAMP.db
    
    # Keep last 30 days
    find $BACKUP_DIR -mtime +30 -delete

Manual Backup

  cp /path/to/octaskly.db /path/to/backup/octaskly_$(date +%s).db

Restore from Backup

  cp /path/to/backup/octaskly_backup.db /path/to/octaskly.db
  systemctl restart octaskly

PERFORMANCE TUNING

Database Optimization

  SQLite Settings:
    PRAGMA journal_mode = WAL;      # Already set
    PRAGMA synchronous = NORMAL;    # Async writes
    PRAGMA cache_size = 10000;      # Larger cache
    PRAGMA temp_store = MEMORY;     # Memory temp

  Query Optimization:
    - Add indexes on frequent queries
    - Use EXPLAIN QUERY PLAN
    - Vacuum database periodically

Worker Pool Tuning

  Optimal Configuration:
    workers = CPU_cores * 2
    max_jobs_per_worker = 4
    connection_pool_size = workers * 2

Network Tuning

  TCP Configuration:
    net.ipv4.tcp_nodelay = 1
    net.ipv4.tcp_tw_reuse = 1
    net.core.somaxconn = 1024

SECURITY HARDENING

Network Isolation

  Firewall Rules:
    - Allow 5555/TCP to trusted networks only
    - Allow 3000/TCP to LAN only
   - Deny all other incoming

System Hardening

  User Privileges:
    - Run octaskly as non-root user
    - Use SELinux contexts
    - Limit file permissions (0750)
  
  File Permissions:
    /etc/octaskly/secret: 0600
    /var/lib/octaskly: 0700
    /opt/octaskly/bin: 0755

Secret Management

  Store secret key using:
    - Environment variable (development)
    - HashiCorp Vault (production)
    - Kubernetes secrets (k8s)
    - Encrypted config files

UPGRADE PROCEDURES

Version 1.0.0

  Prerequisites:
    - Backup current database
    - Review release notes
    - Test in staging
  
  Steps:
    1. Stop current service
    2. Backup database
    3. Build new binary
    4. Run migration (automatic)
    5. Start new service
    6. Verify functionality
  
  Rollback:
    1. Stop service
    2. Restore database backup
    3. Restart with old binary

TROUBLESHOOTING

Service Won't Start

  Check logs:
    journalctl -u octaskly.service -n 50
  
  Common issues:
    - Port already in use: lsof -i :5555
    - Permission denied: check file ownership
    - Database locked: restart service

High Memory Usage

  Investigate:
    - Check active task count
    - Monitor worker pool size
    - Review resource limits
    - Check for memory leaks
  
  Resolution:
    - Reduce worker count
    - Lower resource limits
    - Increase garbage collection
    - Restart service

Task Execution Failures

  Debug steps:
    1. Check dispatcher logs
    2. Check worker logs
    3. Review task stderr
    4. Verify resource limits
    5. Check network connectivity

OPERATIONS CHECKLIST

Daily Tasks
  - Monitor system health
  - Check error logs
  - Verify backup completion

Weekly Tasks
  - Review audit logs
  - Check database size/grow rate
  - Update security policies
  - Test backup restore

Monthly Tasks
  - Rotate secret keys
  - Update dependencies
  - Performance analysis
  - Capacity planning

Quarterly Tasks
  - Security audit
  - Disaster recovery drill
  - Major version updates
  - Architecture review

SUPPORT AND TROUBLESHOOTING

Getting Help

  Documentation:
    - README.md: Quick start
    - SECURITY.md: Security details
    - API_REFERENCE.md: REST API
    - PROJECT_STRUCTURE.md: Code overview
  
  Issues:
    GitHub: https://github.com/adauldev/octaskly/issues
  
  Contact:
    Email: support@example.com
    Response time: 24 hours

Diagnostic Information

  Collect diagnostic data:
    - octaskly version
    - OS/kernel info
    - Rust compiler version
    - System resources
    - Error logs
    - Task command
    - Environment variables
