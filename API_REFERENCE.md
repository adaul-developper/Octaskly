API REFERENCE

REST API Documentation

Base URL
  http://localhost:3000

Authentication
  All endpoints (except /health) require JWT Bearer token

  Header format:
    Authorization: Bearer <JWT_TOKEN>

  Obtain token:
    Generate via dispatcher configuration
    Include user role and permissions
    Token expiration: 24 hours (default)

Error Responses

  Standard error format:
    {
      "error": "Error message",
      "status": 400,
      "timestamp": "2026-02-06T12:00:00Z"
    }

  Common status codes:
    200 - Success
    201 - Created
    400 - Bad request
    401 - Unauthorized
    403 - Forbidden
    404 - Not found
    500 - Internal server error

ENDPOINTS

POST /api/v1/tasks

  Create a new task for execution
  
  Request
    Header: Authorization: Bearer <TOKEN>
    Body:
      {
        "command": "executable command line",
        "timeout": 3600,
        "inputs": ["file1.txt", "file2.txt"],
        "outputs": ["result.txt"]
      }
  
  Parameters
    command (string, required)
      - Shell command to execute
      - Must be a valid command
      - Examples: "echo hello", "cargo build", "python script.py"
    
    timeout (integer, optional)
      - Maximum execution time in seconds
      - Default: 3600 (1 hour)
      - Minimum: 1
      - Maximum: 86400 (24 hours)
    
    inputs (array, optional)
      - Input files/paths needed for execution
      - Relative to worker directory
      - Used for validation only
    
    outputs (array, optional)
      - Expected output files/paths
      - Captured after execution
      - Used for result collection

  Response (201 Created)
    {
      "id": "task-001-a1b2c3d4",
      "command": "echo hello",
      "status": "queued",
      "created_at": "2026-02-06T12:00:00Z"
    }

  Response (400 Bad Request)
    {
      "error": "Invalid command format",
      "status": 400
    }

  Response (401 Unauthorized)
    {
      "error": "Missing or invalid authentication token",
      "status": 401
    }

GET /api/v1/tasks

  List all tasks with optional filtering
  
  Request
    Header: Authorization: Bearer <TOKEN>
    Query parameters:
      - status (queued|running|completed|failed)
      - worker_id (specific worker)
      - limit (default: 100, max: 1000)
      - offset (for pagination)

  Response (200 OK)
    {
      "tasks": [
        {
          "id": "task-001",
          "command": "echo hello",
          "status": "completed",
          "worker_id": "worker-01",
          "exit_code": 0,
          "created_at": "2026-02-06T11:00:00Z",
          "completed_at": "2026-02-06T12:00:00Z"
        }
      ],
      "total": 150,
      "returned": 50
    }

  Query Examples
    List first 50 tasks:
      GET /api/v1/tasks?limit=50
    
    List only completed tasks:
      GET /api/v1/tasks?status=completed
    
    List tasks from specific worker:
      GET /api/v1/tasks?worker_id=worker-01
    
    Pagination:
      GET /api/v1/tasks?limit=100&offset=200

GET /api/v1/tasks/{task-id}

  Retrieve details for specific task
  
  Request
    Header: Authorization: Bearer <TOKEN>
    Path parameter: task-id (required)

  Response (200 OK)
    {
      "id": "task-001-a1b2c3d4",
      "command": "cargo build --release",
      "status": "completed",
      "worker_id": "laptop-01",
      "stdout": "Compiling octaskly...",
      "stderr": "",
      "exit_code": 0,
      "duration_ms": 45000,
      "created_at": "2026-02-06T11:00:00Z",
      "completed_at": "2026-02-06T12:00:00Z"
    }

  Response (404 Not Found)
    {
      "error": "Task not found",
      "status": 404
    }

DELETE /api/v1/tasks/{task-id}

  Cancel a task before execution
  
  Request
    Header: Authorization: Bearer <TOKEN>
    Path parameter: task-id (required)

  Response (200 OK)
    {
      "id": "task-001",
      "status": "cancelled",
      "message": "Task cancelled successfully"
    }

  Response (409 Conflict)
    {
      "error": "Cannot cancel running task",
      "status": 409
    }

  Notes
    - Only queued tasks can be cancelled
    - Running tasks cannot be cancelled (design choice)
    - Completed tasks return 404

GET /api/v1/stats

  Retrieve system statistics and metrics
  
  Request
    Header: Authorization: Bearer <TOKEN>

  Response (200 OK)
    {
      "workers": {
        "total": 3,
        "active": 2,
        "idle": 1
      },
      "tasks": {
        "queued": 5,
        "running": 2,
        "completed": 150,
        "failed": 3
      },
      "system": {
        "uptime_seconds": 86400,
        "memory_usage_mb": 512,
        "cpu_usage_percent": 25.5
      },
      "database": {
        "total_tasks": 160,
        "disk_size_mb": 15
      }
    }

GET /health

  Health check endpoint (no authentication required)
  
  Request
    (No headers required)

  Response (200 OK)
    {
      "status": "healthy",
      "version": "1.0.0",
      "uptime_seconds": 3600
    }

  Response (503 Service Unavailable)
    {
      "status": "unhealthy",
      "message": "Database connection failed"
    }

ERROR CODES

400 Bad Request
  - Invalid command format
  - Missing required field
  - Invalid parameter value
  - Malformed JSON

401 Unauthorized
  - Missing authentication token
  - Invalid token format
  - Token signature mismatch
  - Token expired

403 Forbidden
  - Insufficient permissions
  - User role cannot perform action
  - Access denied

404 Not Found
  - Task does not exist
  - Worker not found
  - Resource deleted

409 Conflict
  - Task already running
  - Worker already registered
  - Duplicate task ID

500 Internal Server Error
  - Database connection failure
  - Unexpected server state
  - System resource exhausted

CLIENT LIBRARIES (RECOMMENDED)

Python Client

  import requests
  
  base_url = "http://localhost:3000"
  token = "bearer_token_here"
  headers = {"Authorization": f"Bearer {token}"}
  
  # Create task
  task_data = {
      "command": "echo hello",
      "timeout": 3600
  }
  response = requests.post(
      f"{base_url}/api/v1/tasks",
      json=task_data,
      headers=headers
  )
  task_id = response.json()["id"]
  
  # Get task status
  response = requests.get(
      f"{base_url}/api/v1/tasks/{task_id}",
      headers=headers
  )
  print(response.json())

cURL Examples

  Create task:
    curl -X POST http://localhost:3000/api/v1/tasks \
      -H "Authorization: Bearer TOKEN" \
      -H "Content-Type: application/json" \
      -d '{"command":"echo test","timeout":3600}'
  
  List tasks:
    curl -X GET http://localhost:3000/api/v1/tasks \
      -H "Authorization: Bearer TOKEN"
  
  Get task details:
    curl -X GET http://localhost:3000/api/v1/tasks/task-001 \
      -H "Authorization: Bearer TOKEN"
  
  Cancel task:
    curl -X DELETE http://localhost:3000/api/v1/tasks/task-001 \
      -H "Authorization: Bearer TOKEN"
  
  Get statistics:
    curl -X GET http://localhost:3000/api/v1/stats \
      -H "Authorization: Bearer TOKEN"
  
  Health check:
    curl -X GET http://localhost:3000/health

Rust Client

  use reqwest::Client;

  Notes
  - The `/health` endpoint returns a JSON payload with `version` and `uptime` fields and can be used by orchestration tools to verify service liveness. Example: `{ "status": "ok", "version": "1.0.0", "uptime_seconds": 12345 }`.
  use serde_json::json;
  
  #[tokio::main]
  async fn main() -> Result<(), Box<dyn std::error::Error>> {
      let client = Client::new();
      let token = "bearer_token";
      
      let task = json!({
          "command": "echo hello",
          "timeout": 3600
      });
      
      let response = client
          .post("http://localhost:3000/api/v1/tasks")
          .header("Authorization", format!("Bearer {}", token))
          .json(&task)
          .send()
          .await?;
      
      println!("{:?}", response.json::<serde_json::Value>().await?);
      Ok(())
  }

RATE LIMITING

Current Implementation
  - No rate limiting (all requests accepted)
  - Planned for future versions

Future Plans
  - Per-token rate limits
  - Sliding window algorithm
  - Configurable limits by role
  - Request queuing

PAGINATION

List Endpoints
  - Support limit and offset parameters
  - Default limit: 100
  - Maximum limit: 1000

Example
  GET /api/v1/tasks?limit=50&offset=200
  
  Returns:
  {
    "tasks": [...],
    "total": 5000,
    "returned": 50,
    "limit": 50,
    "offset": 200
  }

VERSIONING

API Versions
  - Current: v1
  - Endpoint pattern: /api/v1/...
  - Future versions: /api/v2/...

Backward Compatibility
  - v1.x changes are backward compatible
  - Breaking changes only in major versions
  - Deprecation notices provided 6 months advance

RESPONSE HEADERS

Standard Headers
  Content-Type: application/json
  X-Request-ID: unique-request-identifier
  X-Response-Time-ms: execution-time
  
Cache Control
  Cache-Control: no-cache, no-store, must-revalidate
  Pragma: no-cache
  Expires: 0

CORS

Enabled Origins
  - localhost:*
  - 127.0.0.1:*
  - Configurable for deployment

Methods Allowed
  - GET, POST, DELETE
  - OPTIONS (for preflight)

Headers Allowed
  - Content-Type
  - Authorization
  - X-Custom-Header (future)

BEST PRACTICES

Task Design
  - Keep commands simple and focused
  - Use explicit absolute paths
  - Include error handling in scripts
  - Set appropriate timeouts
  - Specify inputs and outputs

Security
  - Rotate tokens regularly
  - Use HTTPS in production
  - Validate all inputs
  - Never commit tokens to version control
  - Monitor audit logs

Performance
  - Reuse HTTP connections
  - Batch task submissions when possible
  - Use appropriate pagination limits
  - Cache worker lists
  - Monitor error rates

Error Handling
  - Implement exponential backoff for retries
  - Log all API errors
  - Handle network timeouts gracefully
  - Validate responses before processing

TROUBLESHOOTING

Connection Refused
  - Verify dispatcher is running
  - Check port number (default: 3000)
  - Check firewall rules
  - Verify network connectivity

Authentication Errors
  - Verify token is valid
  - Check token expiration
  - Verify Bearer token format
  - Check token permissions

Task Failures
  - Review task stderr output
  - Check resource limits
  - Verify command syntax
  - Check worker capacity

Timeout Issues
  - Increase timeout value
  - Check worker performance
  - Monitor system resources
  - Consider task decomposition
