use crate::{
    error::{AppResult, ErrorResponse},
    state::SharedState,
};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
#[allow(unused_imports)] // Used in #[schema(example = json!({...}))] attributes
use serde_json::json;
use std::time::Instant;
use sysinfo::System;
use utoipa::ToSchema;

/// Application start time for uptime calculation
pub static APP_START_TIME: std::sync::OnceLock<Instant> = std::sync::OnceLock::new();

/// Health check response
#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[schema(example = json!({
    "status": "healthy",
    "timestamp": "2024-01-01T00:00:00Z",
    "uptime_seconds": 3600,
    "version": "0.1.0",
    "database": {
        "connected": true,
        "response_time_ms": 5
    },
    "system": {
        "memory_used_mb": 1024,
        "memory_total_mb": 8192,
        "memory_usage_percent": 12.5,
        "cpu_count": 8
    }
}))]
pub struct HealthResponse {
    pub status: HealthStatus,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub uptime_seconds: u64,
    pub version: String,
    pub database: DatabaseHealth,
    pub system: SystemHealth,
}

/// Health status
#[derive(Debug, Serialize, Deserialize, PartialEq, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

/// Database health information
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct DatabaseHealth {
    pub connected: bool,
    pub response_time_ms: Option<u64>,
}

/// System health information
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SystemHealth {
    pub memory_used_mb: u64,
    pub memory_total_mb: u64,
    pub memory_usage_percent: f32,
    pub cpu_count: usize,
}

/// Basic health check endpoint (liveness probe)
#[utoipa::path(
    get,
    path = "/health/live",
    tag = "health",
    responses(
        (status = 200, description = "Service is alive", body = serde_json::Value),
    ),
)]
pub async fn liveness() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "alive",
        "timestamp": chrono::Utc::now(),
    }))
}

/// Readiness check endpoint
#[utoipa::path(
    get,
    path = "/health/ready",
    tag = "health",
    responses(
        (status = 200, description = "Service is ready", body = serde_json::Value),
        (status = 503, description = "Service is not ready", body = serde_json::Value),
    ),
)]
pub async fn readiness(State(state): State<SharedState>) -> impl IntoResponse {
    // Check database connectivity
    let db_healthy = state.db.health_check().await.is_ok();

    if db_healthy {
        (
            StatusCode::OK,
            Json(serde_json::json!({
                "status": "ready",
                "timestamp": chrono::Utc::now(),
            })),
        )
    } else {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({
                "status": "not_ready",
                "timestamp": chrono::Utc::now(),
                "reason": "database_unavailable",
            })),
        )
    }
}

/// Comprehensive health check endpoint
#[utoipa::path(
    get,
    path = "/health",
    tag = "health",
    responses(
        (status = 200, description = "Service health information", body = HealthResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
)]
pub async fn health_check(State(state): State<SharedState>) -> AppResult<impl IntoResponse> {
    let start_time = APP_START_TIME.get_or_init(Instant::now);
    let uptime = start_time.elapsed().as_secs();

    // Check database health
    let db_start = Instant::now();
    let db_connected = state.db.health_check().await.is_ok();
    let db_response_time = if db_connected {
        Some(db_start.elapsed().as_millis() as u64)
    } else {
        None
    };

    // Get system information
    let mut sys = System::new_all();
    sys.refresh_memory();

    let memory_used = sys.used_memory() / 1024 / 1024; // Convert to MB
    let memory_total = sys.total_memory() / 1024 / 1024; // Convert to MB
    let memory_usage_percent = (memory_used as f32 / memory_total as f32) * 100.0;
    let cpu_count = num_cpus::get();

    // Determine overall health status
    let status = if !db_connected {
        HealthStatus::Unhealthy
    } else if memory_usage_percent > 90.0 {
        HealthStatus::Degraded
    } else {
        HealthStatus::Healthy
    };

    let response = HealthResponse {
        status,
        timestamp: chrono::Utc::now(),
        uptime_seconds: uptime,
        version: env!("CARGO_PKG_VERSION").to_string(),
        database: DatabaseHealth {
            connected: db_connected,
            response_time_ms: db_response_time,
        },
        system: SystemHealth {
            memory_used_mb: memory_used,
            memory_total_mb: memory_total,
            memory_usage_percent,
            cpu_count,
        },
    };

    Ok(Json(response))
}
