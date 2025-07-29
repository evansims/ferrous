use crate::{
    error::{AppResult, ErrorResponse},
    models::{CreateItemRequest, Item, UpdateItemRequest},
    state::SharedState,
    validation::ValidatedJson,
};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use validator::Validate;

/// Query parameters for listing items
#[derive(Debug, Deserialize, Validate, IntoParams)]
pub struct ListQuery {
    /// Maximum number of items to return (1-100)
    #[validate(range(min = 1, max = 100, message = "Limit must be between 1 and 100"))]
    #[param(minimum = 1, maximum = 100, example = 20)]
    pub limit: Option<usize>,

    /// Number of items to skip
    #[validate(range(min = 0, message = "Offset must be non-negative"))]
    #[param(minimum = 0, example = 0)]
    pub offset: Option<usize>,
}

/// Paginated list response
#[derive(Debug, Serialize, ToSchema)]
pub struct ListResponse<T> {
    /// List of items
    pub items: Vec<T>,
    /// Total number of items available
    pub total: usize,
    /// Number of items returned
    pub limit: usize,
    /// Number of items skipped
    pub offset: usize,
}

/// List all items with pagination
#[utoipa::path(
    get,
    path = "/api/v1/items",
    tag = "items",
    params(
        ListQuery
    ),
    responses(
        (status = 200, description = "List of items", body = ListResponse<Item>),
        (status = 400, description = "Invalid query parameters", body = ErrorResponse),
    ),
)]
pub async fn list_items(
    Query(params): Query<ListQuery>,
    State(state): State<SharedState>,
) -> AppResult<impl IntoResponse> {
    // Validate query parameters
    params.validate()?;

    let limit = params.limit.unwrap_or(20).min(100);
    let offset = params.offset.unwrap_or(0);

    let items = state.db.items().list(limit, offset).await?;
    let total = state.db.items().count().await?;

    Ok(Json(ListResponse {
        items,
        total,
        limit,
        offset,
    }))
}

/// Create a new item
#[utoipa::path(
    post,
    path = "/api/v1/items",
    tag = "items",
    request_body = CreateItemRequest,
    responses(
        (status = 201, description = "Item created successfully", body = Item),
        (status = 400, description = "Invalid request", body = ErrorResponse),
        (status = 422, description = "Validation error", body = ErrorResponse),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn create_item(
    State(state): State<SharedState>,
    ValidatedJson(payload): ValidatedJson<CreateItemRequest>,
) -> AppResult<impl IntoResponse> {
    let sanitized_payload = payload.sanitize();
    let item = state.db.items().create(sanitized_payload).await?;
    Ok((StatusCode::CREATED, Json(item)))
}

/// Get a specific item by ID
#[utoipa::path(
    get,
    path = "/api/v1/items/{id}",
    tag = "items",
    params(
        ("id" = String, Path, description = "Item ID")
    ),
    responses(
        (status = 200, description = "Item found", body = Item),
        (status = 404, description = "Item not found", body = ErrorResponse),
    ),
)]
pub async fn get_item(
    Path(id): Path<String>,
    State(state): State<SharedState>,
) -> AppResult<impl IntoResponse> {
    let item = state.db.items().get(&id).await?;
    Ok(Json(item))
}

/// Update an existing item
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
        (status = 404, description = "Item not found", body = ErrorResponse),
        (status = 422, description = "Validation error", body = ErrorResponse),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn update_item(
    Path(id): Path<String>,
    State(state): State<SharedState>,
    ValidatedJson(payload): ValidatedJson<UpdateItemRequest>,
) -> AppResult<impl IntoResponse> {
    let sanitized_payload = payload.sanitize();
    let item = state.db.items().update(&id, sanitized_payload).await?;
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
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn delete_item(
    Path(id): Path<String>,
    State(state): State<SharedState>,
) -> AppResult<impl IntoResponse> {
    state.db.items().delete(&id).await?;
    Ok(StatusCode::NO_CONTENT)
}
