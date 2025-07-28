use crate::{
    error::{AppError, AppResult},
    models::{CreateItemRequest, Item, UpdateItemRequest},
    state::SharedState,
};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

#[derive(Debug, Serialize)]
pub struct ListResponse<T> {
    pub items: Vec<T>,
    pub total: usize,
    pub limit: usize,
    pub offset: usize,
}

pub async fn list_items(
    Query(params): Query<ListQuery>,
    State(state): State<SharedState>,
) -> AppResult<impl IntoResponse> {
    let items = state.items.read().map_err(|_| AppError::LockError)?;

    let limit = params.limit.unwrap_or(20).min(100);
    let offset = params.offset.unwrap_or(0);

    let all_items: Vec<Item> = items.values().cloned().collect();
    let total = all_items.len();
    let paginated_items = all_items.into_iter().skip(offset).take(limit).collect();

    Ok(Json(ListResponse {
        items: paginated_items,
        total,
        limit,
        offset,
    }))
}

pub async fn create_item(
    State(state): State<SharedState>,
    Json(payload): Json<CreateItemRequest>,
) -> AppResult<impl IntoResponse> {
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now();

    let item = Item {
        id: id.clone(),
        name: payload.name,
        description: payload.description,
        created_at: now,
        updated_at: now,
    };

    let mut items = state.items.write().map_err(|_| AppError::LockError)?;

    items.insert(id, item.clone());

    Ok((StatusCode::CREATED, Json(item)))
}

pub async fn get_item(
    Path(id): Path<String>,
    State(state): State<SharedState>,
) -> AppResult<impl IntoResponse> {
    let items = state.items.read().map_err(|_| AppError::LockError)?;

    items
        .get(&id)
        .cloned()
        .map(Json)
        .ok_or_else(|| AppError::NotFound(format!("Item with id '{}' not found", id)))
}

pub async fn update_item(
    Path(id): Path<String>,
    State(state): State<SharedState>,
    Json(payload): Json<UpdateItemRequest>,
) -> AppResult<impl IntoResponse> {
    let mut items = state.items.write().map_err(|_| AppError::LockError)?;

    let item = items
        .get_mut(&id)
        .ok_or_else(|| AppError::NotFound(format!("Item with id '{}' not found", id)))?;

    if let Some(name) = payload.name {
        item.name = name;
    }
    if payload.description.is_some() {
        item.description = payload.description;
    }
    item.updated_at = chrono::Utc::now();

    Ok(Json(item.clone()))
}

pub async fn delete_item(
    Path(id): Path<String>,
    State(state): State<SharedState>,
) -> AppResult<impl IntoResponse> {
    let mut items = state.items.write().map_err(|_| AppError::LockError)?;

    items
        .remove(&id)
        .ok_or_else(|| AppError::NotFound(format!("Item with id '{}' not found", id)))?;

    Ok(StatusCode::NO_CONTENT)
}
