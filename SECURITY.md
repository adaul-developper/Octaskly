SECURITY DOCUMENTATION

Security Model Overview

Octaskly implements defense-in-depth security with multiple layers:

  1. Transport Security (Encryption)
  2. Authentication & Authorization
  3. Task Isolation (Sandboxing)
  4. Resource Control (Limits)
  5. Audit Logging

Each layer provides independent security guarantees.

ENCRYPTION

AES-256-GCM Standard

  Algorithm: AES with 256-bit keys in GCM mode
  Key Size: 256 bits (32 bytes)
  Nonce: 96 bits (12 bytes, randomly generated per message)
  Tag: 128 bits (16 bytes, authentication)
  
  Characteristics:
    - Symmetric encryption (same key for encrypt/decrypt)
    - Authenticated encryption (detects tampering)
    - AEAD (Authenticated Encryption with Associated Data)
    - Constant-time operations (resistant to timing attacks)

Key Derivation

  Process:
    Input password -> SHA-256 hash -> AES-256 key
  
  Implementation:
    pub fn derive_key(password: &str) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(password.as_bytes());
        let result = hasher.finalize();
        let mut key = [0u8; 32];
        key.copy_from_slice(&result[..32]);
        key
    }

  Security Notes:
    - Use strong passwords (16+ characters)
    - Include uppercase, lowercase, numbers, symbols
    - Do not reuse passwords across systems
    - Rotate keys periodically (monthly recommended)

Message Encryption

  All messages between nodes are encrypted:
    1. Serialize message (bincode)
    2. Generate random nonce
    3. Encrypt payload with AES-256-GCM
    4. Send nonce + ciphertext + tag
  
  Decryption process reverses:
    1. Extract nonce
    2. Decrypt ciphertext
    3. Verify authentication tag
    4. Deserialize message

Token Security

  HMAC-SHA256 Based

  Process:
    Input data -> HMAC key -> Hash -> Token
  
  Verification:
    Recalculate hash -> Compare with token
  
  Use Cases:
    - Worker authentication
    - Message integrity
    - Timestamp verification

AUTHENTICATION

JWT Token System

  Standard: RFC 7519 (JSON Web Tokens)
  
  Token Structure:
    header.payload.signature
  
  Header:
    {
      "alg": "HS256",
      "typ": "JWT"
    }
  
  Payload (Claims):
    {
      "sub": "username",
      "role": "dispatcher",
      "exp": 1708867200,
      "iat": 1708780800,
      "permissions": ["read", "write"]
    }
  
  Signature:
    HMAC-SHA256(header + payload, secret_key)

Token Lifecycle

  Generation:
    Issued when user authenticates
    Contains user identity and permissions
    Signed with system secret key
    
  Usage:
    Sent in HTTP Authorization header
    Verified on each API request
    Checked for expiration
    
  Expiration:
    Default: 24 hours
    Can be customized per role
    Refresh mechanism (future)

Token Security Best Practices

  Storage:
    - Never log or expose tokens
    - Store in memory only
    - Use secure configuration management
    - Rotate regularly
  
  Transmission:
    - Always use HTTPS (TLS)
    - Send only in Authorization header
    - Never include in URL parameters
    - Never include in cookies
  
  Validation:
    - Verify signature always
    - Check expiration time
    - Verify claimed role/permissions
    - Validate custom claims

Installer & Release Integrity

  - All installers and release artifacts are published on GitHub Releases over HTTPS.
  - Verify downloaded artifacts using SHA256 checksums provided in the release (`SHA256SUMS.txt`).
  - Prefer the one-line installer which performs checksum verification when possible.
  - For high-security environments, validate release tags and verify commits/signatures before deployment.

AUTHORIZATION

Role-Based Access Control (RBAC)

  Four Roles Defined:

  Admin
    - Full system access
    - Manage users and roles
    - Configure security parameters
    - Access all resources
    - Permissions: ["admin", "dispatcher", "worker", "client"]
  
  Dispatcher
    - Task and worker management
    - Submit and monitor tasks
    - View worker status
    - Access task results
    - Permissions: ["dispatcher", "client"]
  
  Worker
    - Execute assigned tasks
    - Report task results
    - Register with dispatcher
    - Permissions: ["worker"]
  
  Client
    - Submit tasks
    - View own task results
    - Permissions: ["client"]

Permission Matrix

  Resource      | Admin | Dispatcher | Worker | Client
  --------------|-------|-----------|--------|-------
  Create Task   | Yes   | Yes       | No     | Yes
  List Tasks    | Yes   | Yes       | No     | Own only
  Cancel Task   | Yes   | Yes       | No     | Own only
  Get Stats     | Yes   | Yes       | No     | No
  Register Work | Yes   | Yes       | Yes    | No
  Shutdown      | Yes   | No        | No     | No

Permission Checking

  Implementation:
    let permitted = auth.has_permission(&claims, "task:create")?;
    if !permitted {
        return Err("Insufficient permissions");
    }

SANDBOXING

Isolation Levels

  Level None
    - No isolation applied
    - Full system access
    - Use only for trusted environments
  
  Level Basic
    - Clear environment variables
    - Restricted PATH (no system directories)
    - Standard /tmp access
    - Use for semi-trusted tasks
  
  Level Strict
    - Only /tmp accessible
    - Minimal environment
    - No shell access
    - No system utilities
    - Use for untrusted code
  
  Level VeryStrict
    - Empty environment
    - No write permissions
    - Read-only execution
    - Maximum isolation
    - Use for hostile code

Implementation Details

  Process Isolation:
    - Run in separate process group
    - Resource limits via rlimit
    - File descriptor limits
    - Process count limits
  
  Environment Control:
    - Clear inherited environment
    - Whitelist specific variables
    - Set secure defaults
  
  File System:
    - Allowlist specific paths
    - Deny access to system files
    - Restrict temporary directory
    - Prevent symlink traversal

RESOURCE LIMITS

Per-Task Limit Types Supported

  CPU Time
    - Maximum execution time (seconds)
    - Enforced via timeout mechanism
    - Default: 3600 seconds (1 hour)
  
  Memory
    - Maximum RAM usage (MB)
    - Enforced via rlimit RLIMIT_AS
    - Default: 2048 MB
  
  Disk Usage
    - Maximum disk space (MB)
    - Enforced via rlimit RLIMIT_FSIZE
    - Default: 10000 MB
  
  Open Files
    - Maximum file descriptors
    - Enforced via rlimit RLIMIT_NOFILE
    - Default: 1024
  
  Processes
    - Maximum child processes
    - Enforced via rlimit RLIMIT_NPROC
    - Default: 256

Preset Configurations

  Default Limits (for general tasks):
    CPU: 3600 seconds (1 hour)
    Memory: 2048 MB
    Disk: 10000 MB
    Files: 1024
    Processes: 256
  
  Strict Limits (for untrusted code):
    CPU: 300 seconds (5 minutes)
    Memory: 512 MB
    Disk: 1000 MB
    Files: 100
    Processes: 10
  
  Relaxed Limits (for long builds):
    CPU: 86400 seconds (24 hours)
    Memory: 8192 MB
    Disk: 102400 MB
    Files: 4096
    Processes: 1024

Limit Enforcement Mechanism

  Pre-Execution:
    1. Create child process
    2. Apply rlimit settings
    3. Execute task command
  
  During Execution:
    - Monitor resource usage
    - Check against limits
    - Trigger warning at 80%
    - Kill process at 100%
  
  Post-Execution:
    - Report resource usage
    - Store metrics in database
    - Generate usage statistics

AUDIT LOGGING

Logged Events

  System Events
    - Node startup/shutdown
    - Configuration changes
    - Database operations
    - Security failures
  
  Task Events
    - Task created
    - Task assigned
    - Task started
    - Task completed
    - Task failed
    - Task cancelled
  
  Worker Events
    - Worker registered
    - Worker deregistered
    - Worker status changed
    - Worker heartbeat lost
  
  Security Events
    - Authentication success/failure
    - Authorization denial
    - Token generation
    - Token expiration
    - Encryption enabled/disabled
    - Whitelist changes

Log Format

  {
    "timestamp": "2026-02-06T12:00:00Z",
    "event_type": "task_completed",
    "worker_id": "worker-001",
    "task_id": "task-001-a1b2c3d4",
    "details": {
      "status": "success",
      "exit_code": 0,
      "duration_ms": 45000,
      "memory_used_mb": 512
    },
    "level": "info"
  }

Log Storage

  Location: SQLite audit_log table
  Retention: Configurable (default: 90 days)
  Rotation: Automatic via cleanup_old_tasks()
  Backup: Included in database backups

Log Query Examples

  Get security events:
    SELECT * FROM audit_log
    WHERE event_type LIKE 'auth%'
    ORDER BY timestamp DESC
    LIMIT 100;
  
  Get failed tasks:
    SELECT * FROM audit_log
    WHERE event_type = 'task_failed'
    AND timestamp > datetime('now', '-1 day');
  
  Get worker activity:
    SELECT * FROM audit_log
    WHERE worker_id = 'worker-001'
    ORDER BY timestamp DESC;

WHITELIST MANAGEMENT

Worker Whitelisting

  Purpose:
    - Control which workers can connect
    - Prevent unauthorized participation
    - Implement network segmentation
  
  Configuration:
    workers_whitelist = [
      "worker-001",
      "worker-002",
      "trusted-device"
    ]
  
  Behavior:
    - Empty list: All workers allowed (warning logged)
    - Non-empty list: Only listed workers allowed
    - Attempt to join: Rejected if not whitelisted
    - Logging: All attempts logged

Implementation

  pub struct SecurityManager {
      whitelist: Vec<String>,
  }
  
  impl SecurityManager {
      pub fn add_to_whitelist(&mut self, worker_id: String) {
          self.whitelist.push(worker_id);
      }
      
      pub fn is_whitelisted(&self, worker_id: &str) -> bool {
          self.whitelist.is_empty() || 
          self.whitelist.contains(&worker_id.to_string())
      }
  }

BEST PRACTICES

For Operators

  1. Key Management
     - Use strong, random secret keys
     - Rotate keys on schedule
     - Store keys securely (separate from code)
     - Use environment variables
     - Never commit to version control
  
  2. Network Security
     - Use HTTPS/TLS (production)
     - Restrict network access
     - Use VPN if cross-network
     - Monitor network traffic
     - Enable firewall rules
  
  3. User Management
     - Assign least privilege roles
     - Monitor user activity
     - Revoke unused credentials
     - Enforce password policies
     - Use single sign-on (future)
  
  4. Monitoring
     - Review audit logs regularly
     - Set up alerts for failures
     - Monitor resource usage
     - Watch for suspicious activity
     - Keep system updated

For Application Developers

  1. Task Design
     - Avoid shell injection (use exec.Command)
     - Validate all inputs
     - Use absolute paths
     - Don't trust user data
     - Set appropriate timeouts
  
  2. Error Handling
     - Log security-relevant errors
     - Don't expose internal details
     - Use generic error messages
     - Include context for debugging
  
  3. Resource Management
     - Set realistic resource limits
     - Monitor long-running tasks
     - Clean up after failures
     - Test resource enforcement
  
  4. Testing
     - Test with untrusted input
     - Verify isolation works
     - Check permission enforcement
     - Audit log verification

THREAT MODEL

Assumptions

  Network is local (LAN/WiFi)
  Participants may be semi-trusted
  Attacker has network access
  No physical access to nodes

Threats Considered

  Task Injection
    - Mitigation: Input validation, sandboxing
  
  Information Disclosure
    - Mitigation: Encryption, access control
  
  Privilege Escalation
    - Mitigation: RBAC, resource limits
  
  Denial of Service
    - Mitigation: Resource limits, rate limiting
  
  Man-in-the-Middle
    - Mitigation: Encryption, authentication

Threats Out of Scope

  - Physical node compromise
  - Cryptographic key extraction
  - System-level attacks
  - Supply chain attacks

SECURITY ROADMAP

Near-term (0.3.0)
  - Rate limiting
  - TLS support
  - Session management
  - Token refresh mechanism

Medium-term (0.4.0)
  - Hardware security module integration
  - OAuth 2.0 support
  - Advanced audit analytics
  - Encryption key rotation

Long-term (1.0.0)
  - Zero-trust architecture
  - Distributed trust model
  - Advanced threat detection
  - Compliance certifications (SOC2)

INCIDENT RESPONSE

Security Incident Procedure

  1. Detection
     - Monitor logs for anomalies
     - Check audit trail
     - Verify system health
  
  2. Containment
     - Isolate affected nodes
     - Revoke compromised credentials
     - Stop suspicious tasks
  
  3. Investigation
     - Collect logs
     - Analyze audit trail
     - Determine scope
  
  4. Recovery
     - Restore from backup
     - Reset credentials
     - Update security settings
     - Restart services

Reporting

  Security vulnerabilities:
    Email: security@example.com
    Response time: 24 hours
    Do not disclose publicly

COMPLIANCE

Standards Alignment

  OWASP Top 10
    - Injection: Input validation
    - Authentication: JWT tokens
    - Access Control: RBAC
    - Encryption: AES-256-GCM
    - Logging: Audit trail

  Data Protection
    - User data encryption
    - Secure deletion (future)
    - Privacy by design
    - Data retention policies

VERSION INFORMATION

Security Features by Version

  0.1.0
    - Basic pre-shared key
    - Worker whitelisting
  
  1.0.0 (Current)
    - AES-256-GCM encryption
    - JWT authentication
    - RBAC with 4 roles
    - Sandboxing (4 levels)
    - Comprehensive audit logging
    - Resource limits enforcement
