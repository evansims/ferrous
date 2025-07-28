use crate::{
    database::DatabaseResult,
    models::{CreateItemRequest, Item, UpdateItemRequest},
};
use async_trait::async_trait;

/// Repository trait for item operations
#[async_trait]
pub trait ItemRepository: Send + Sync {
    /// List all items with pagination
    async fn list(&self, limit: usize, offset: usize) -> DatabaseResult<Vec<Item>>;
    
    /// Get total count of items
    async fn count(&self) -> DatabaseResult<usize>;
    
    /// Get a single item by ID
    async fn get(&self, id: &str) -> DatabaseResult<Item>;
    
    /// Create a new item
    async fn create(&self, request: CreateItemRequest) -> DatabaseResult<Item>;
    
    /// Update an existing item
    async fn update(&self, id: &str, request: UpdateItemRequest) -> DatabaseResult<Item>;
    
    /// Delete an item
    async fn delete(&self, id: &str) -> DatabaseResult<()>;
}