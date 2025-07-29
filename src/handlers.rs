use crate::{
    error::{AppResult, ErrorResponse},
    metrics::get_metrics,
    models::{CreateItemRequest, Item, UpdateItemRequest},
    state::SharedState,
    validation::ValidatedJson,
};
use axum::{
    extract::{Path, Query, State},
    http::{header::CONTENT_TYPE, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
#[allow(unused_imports)] // Used in #[schema(example = json!({...}))] attributes
use serde_json::json;
use std::time::Instant;
use sysinfo::System;
use utoipa::{IntoParams, ToSchema};
use validator::Validate;

// ===== HEALTH CHECK HANDLERS =====

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
    pub timestamp: DateTime<Utc>,
    pub uptime_seconds: u64,
    pub version: String,
    pub database: DatabaseHealth,
    pub system: SystemHealth,
}

/// Health status
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
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
    Json(json!({
        "status": "alive",
        "timestamp": Utc::now(),
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
    let db_healthy = state.repo.health_check().await.is_ok();

    if db_healthy {
        (
            StatusCode::OK,
            Json(json!({
                "status": "ready",
                "timestamp": Utc::now(),
            })),
        )
    } else {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({
                "status": "not_ready",
                "timestamp": Utc::now(),
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
    let db_connected = state.repo.health_check().await.is_ok();
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
        timestamp: Utc::now(),
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

// ===== ITEM HANDLERS =====

/// Query parameters for listing items
#[derive(Debug, Deserialize, Validate, IntoParams)]
pub struct ListQuery {
    #[serde(default = "default_limit")]
    #[validate(range(min = 1, max = 100))]
    pub limit: usize,

    #[serde(default)]
    pub offset: usize,
}

const fn default_limit() -> usize {
    20
}

/// Response for list operations
#[derive(Debug, Serialize, ToSchema)]
#[schema(example = json!({
    "items": [{
        "id": "550e8400-e29b-41d4-a716-446655440000",
        "name": "Example Item",
        "description": "Example description",
        "created_at": "2024-01-01T00:00:00Z",
        "updated_at": "2024-01-01T00:00:00Z"
    }],
    "total": 100,
    "limit": 20,
    "offset": 0
}))]
pub struct ListResponse {
    pub items: Vec<Item>,
    pub total: usize,
    pub limit: usize,
    pub offset: usize,
}

/// Create a new item
#[utoipa::path(
    post,
    path = "/api/v1/items",
    tag = "items",
    request_body = CreateItemRequest,
    responses(
        (status = 201, description = "Item created successfully", body = Item),
        (status = 400, description = "Bad request", body = ErrorResponse),
        (status = 422, description = "Validation error", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
)]
pub async fn create_item(
    State(state): State<SharedState>,
    ValidatedJson(request): ValidatedJson<CreateItemRequest>,
) -> AppResult<impl IntoResponse> {
    let item = state.repo.create(request).await?;
    Ok((StatusCode::CREATED, Json(item)))
}

/// Get an item by ID
#[utoipa::path(
    get,
    path = "/api/v1/items/{id}",
    tag = "items",
    params(
        ("id" = String, Path, description = "Item ID")
    ),
    responses(
        (status = 200, description = "Item retrieved successfully", body = Item),
        (status = 404, description = "Item not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
)]
pub async fn get_item(
    State(state): State<SharedState>,
    Path(id): Path<String>,
) -> AppResult<impl IntoResponse> {
    let item = state.repo.get(&id).await?;
    Ok(Json(item))
}

/// Update an item
#[utoipa::path(
    put,
    path = "/api/v1/items/{id}",
    tag = "items",
    params(
        ("id" = String, Path, description = "Item ID")
    ),
    request_body = UpdateItemRequest,
    responses(
        (status = 200, description = "Item updated successfully", body = Item),
        (status = 400, description = "Bad request", body = ErrorResponse),
        (status = 404, description = "Item not found", body = ErrorResponse),
        (status = 422, description = "Validation error", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
)]
pub async fn update_item(
    State(state): State<SharedState>,
    Path(id): Path<String>,
    ValidatedJson(request): ValidatedJson<UpdateItemRequest>,
) -> AppResult<impl IntoResponse> {
    let item = state.repo.update(&id, request).await?;
    Ok(Json(item))
}

/// Delete an item
#[utoipa::path(
    delete,
    path = "/api/v1/items/{id}",
    tag = "items",
    params(
        ("id" = String, Path, description = "Item ID")
    ),
    responses(
        (status = 204, description = "Item deleted successfully"),
        (status = 404, description = "Item not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
)]
pub async fn delete_item(
    State(state): State<SharedState>,
    Path(id): Path<String>,
) -> AppResult<impl IntoResponse> {
    state.repo.delete(&id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// List items with pagination
#[utoipa::path(
    get,
    path = "/api/v1/items",
    tag = "items",
    params(ListQuery),
    responses(
        (status = 200, description = "Items retrieved successfully", body = ListResponse),
        (status = 400, description = "Bad request", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
)]
pub async fn list_items(
    State(state): State<SharedState>,
    Query(query): Query<ListQuery>,
) -> AppResult<impl IntoResponse> {
    let items = state.repo.list(query.limit, query.offset).await?;
    let total = state.repo.count().await?;

    let response = ListResponse {
        items,
        total,
        limit: query.limit,
        offset: query.offset,
    };

    Ok(Json(response))
}

// ===== METRICS HANDLER =====

/// Prometheus metrics endpoint
pub async fn metrics_handler() -> Result<Response, StatusCode> {
    let metrics = get_metrics();

    Ok((StatusCode::OK, [(CONTENT_TYPE, "text/plain; version=0.0.4")], metrics).into_response())
}
