use crate::{
    error::AppResult,
    models::{CreateItemRequest, UpdateItemRequest},
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

pub async fn create_item(
    State(state): State<SharedState>,
    Json(payload): Json<CreateItemRequest>,
) -> AppResult<impl IntoResponse> {
    let item = state.db.items().create(payload).await?;
    Ok((StatusCode::CREATED, Json(item)))
}

pub async fn get_item(
    Path(id): Path<String>,
    State(state): State<SharedState>,
) -> AppResult<impl IntoResponse> {
    let item = state.db.items().get(&id).await?;
    Ok(Json(item))
}

pub async fn update_item(
    Path(id): Path<String>,
    State(state): State<SharedState>,
    Json(payload): Json<UpdateItemRequest>,
) -> AppResult<impl IntoResponse> {
    let item = state.db.items().update(&id, payload).await?;
    Ok(Json(item))
}

pub async fn delete_item(
    Path(id): Path<String>,
    State(state): State<SharedState>,
) -> AppResult<impl IntoResponse> {
    state.db.items().delete(&id).await?;
    Ok(StatusCode::NO_CONTENT)
}
