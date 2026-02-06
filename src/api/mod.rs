use axum::{
    extract::{Path, State, Json},
    http::StatusCode,
    middleware::Next,
    response::Response,
    routing::{get, post},
    Router,
    http::Request,
    body::Body,
};
use serde_json::json;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use tower_http::cors::CorsLayer;
use anyhow::Result;

use crate::protocol::Task;
use crate::scheduler::Scheduler;
use crate::state::DispatcherState;
use crate::auth::AuthManager;
use crate::persistence::PersistentStore;

#[derive(Clone)]
pub struct ApiState {
    pub scheduler: Arc<Scheduler>,
    pub dispatcher: Arc<DispatcherState>,
    pub auth: Arc<AuthManager>,
    pub store: Arc<PersistentStore>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateTaskRequest {
    pub command: String,
    pub timeout: Option<u64>,
    pub inputs: Option<Vec<String>>,
    pub outputs: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TaskResponse {
    pub id: String,
    pub command: String,
    pub status: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthResponse {
    pub token: String,
    pub expires_in: i64,
}

/// Middleware for JWT verification
#[allow(dead_code)]
async fn auth_middleware(
    State(state): State<ApiState>,
    request: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let auth_header = request
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .map(|h| h.to_string());

    if let Some(header) = auth_header {
        if let Some(token) = header.strip_prefix("Bearer ") {
            match state.auth.verify_token(token) {
                Ok(_claims) => {
                    // Token is valid, proceed
                    return Ok(next.run(request).await);
                }
                Err(_) => {
                    return Err(StatusCode::UNAUTHORIZED);
                }
            }
        }
    }

    Err(StatusCode::UNAUTHORIZED)
}

/// Create a new task
async fn create_task(
    State(state): State<ApiState>,
    Json(req): Json<CreateTaskRequest>,
) -> Result<(StatusCode, Json<TaskResponse>), (StatusCode, String)> {
    let mut task = Task::new(req.command.clone());
    
    if let Some(inputs) = req.inputs {
        task.inputs = inputs;
    }
    if let Some(outputs) = req.outputs {
        task.outputs = outputs;
    }
    if let Some(timeout) = req.timeout {
        task.timeout = timeout;
    }

    state.scheduler.enqueue(task.clone()).await;
    
    let response = TaskResponse {
        id: task.id,
        command: task.command,
        status: "Pending".to_string(),
        created_at: chrono::Local::now().to_rfc3339(),
    };

    Ok((StatusCode::CREATED, Json(response)))
}

/// Get task details
async fn get_task(
    State(state): State<ApiState>,
    Path(task_id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    match state.store.get_task(&task_id) {
        Ok(Some(task)) => Ok(Json(json!({
            "id": task.id,
            "command": task.command,
            "status": task.status,
            "worker_id": task.worker_id,
            "stdout": task.stdout,
            "stderr": task.stderr,
            "exit_code": task.exit_code,
            "duration_ms": task.duration_ms,
            "created_at": task.created_at,
            "completed_at": task.completed_at,
        }))),
        Ok(None) => Err((StatusCode::NOT_FOUND, "Task not found".to_string())),
        Err(_) => Err((StatusCode::INTERNAL_SERVER_ERROR, "Database error".to_string())),
    }
}

/// List all tasks
async fn list_tasks(
    State(state): State<ApiState>,
) -> Result<Json<Vec<serde_json::Value>>, (StatusCode, String)> {
    match state.store.get_all_tasks() {
        Ok(tasks) => {
            let response = tasks
                .iter()
                .map(|t| {
                    json!({
                        "id": t.id,
                        "command": t.command,
                        "status": t.status,
                        "worker_id": t.worker_id,
                        "created_at": t.created_at,
                        "completed_at": t.completed_at,
                    })
                })
                .collect();
            Ok(Json(response))
        }
        Err(_) => Err((StatusCode::INTERNAL_SERVER_ERROR, "Database error".to_string())),
    }
}

/// Cancel a task
async fn cancel_task(
    State(_state): State<ApiState>,
    Path(task_id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    // TODO: Implement task cancellation
    // For now, just return OK
    println!("Cancel task: {}", task_id);
    Ok(StatusCode::OK)
}

/// Get dispatcher stats
async fn get_stats(
    State(state): State<ApiState>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let workers = state.scheduler.get_workers().await;
    let queue_size = state.scheduler.queue_size().await;
    
    match state.store.get_stats() {
        Ok((total, completed, failed)) => {
            Ok(Json(json!({
                "workers_count": workers.len(),
                "idle_workers": workers.iter().filter(|w| w.is_idle()).count(),
                "task_queue": queue_size,
                "total_tasks": total,
                "completed_tasks": completed,
                "failed_tasks": failed,
            })))
        }
        Err(_) => Err((StatusCode::INTERNAL_SERVER_ERROR, "Stats error".to_string())),
    }
}

/// Health check
async fn health_check() -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    Ok(Json(json!({
        "status": "healthy",
        "version": "1.0.0",
    })))
}

/// Create API router
pub fn create_router(state: ApiState) -> Router {
    Router::new()
        // Public endpoints
        .route("/health", get(health_check))
        
        // Task endpoints
        .route("/api/v1/tasks", post(create_task).get(list_tasks))
        .route("/api/v1/tasks/:id", get(get_task).delete(cancel_task))
        
        // Stats endpoint
        .route("/api/v1/stats", get(get_stats))
        
        .with_state(state)
        .layer(CorsLayer::permissive())
}

/// Start REST API server
pub async fn start_api_server(
    addr: &str,
    state: ApiState,
) -> Result<()> {
    let app = create_router(state);
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!("REST API listening on http://{}", addr);
    
    axum::serve(listener, app).await?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_task_request() {
        let req = CreateTaskRequest {
            command: "echo test".to_string(),
            timeout: Some(60),
            inputs: None,
            outputs: None,
        };
        
        assert_eq!(req.command, "echo test");
        assert_eq!(req.timeout, Some(60));
    }
}
