use async_trait::async_trait;
use chrono::Utc;
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};
use uuid::Uuid;

use crate::{
    config::Config,
    metrics::{
        track_database_query, track_item_created, track_item_deleted, track_item_updated, Timer,
        DATABASE_CONNECTIONS,
    },
    models::{CreateItemRequest, Item, UpdateItemRequest},
};

/// Database errors that can occur across all implementations
#[derive(Debug, thiserror::Error, Clone)]
pub enum DatabaseError {
    #[error("Item not found")]
    NotFound,

    #[error("Database connection error: {0}")]
    ConnectionError(String),

    #[error("Database query error: {0}")]
    QueryError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Lock error")]
    LockError,
}

pub type DatabaseResult<T> = Result<T, DatabaseError>;

/// Main repository trait for items
#[async_trait]
pub trait ItemRepository: Send + Sync {
    async fn create(&self, request: CreateItemRequest) -> DatabaseResult<Item>;
    async fn get(&self, id: &str) -> DatabaseResult<Item>;
    async fn update(&self, id: &str, request: UpdateItemRequest) -> DatabaseResult<Item>;
    async fn delete(&self, id: &str) -> DatabaseResult<()>;
    async fn list(&self, limit: usize, offset: usize) -> DatabaseResult<Vec<Item>>;
    async fn count(&self) -> DatabaseResult<usize>;
    async fn health_check(&self) -> DatabaseResult<()>;
}

/// In-memory implementation of the repository
pub struct InMemoryRepository {
    data: Arc<RwLock<HashMap<String, Item>>>,
}

impl InMemoryRepository {
    pub fn new() -> Self {
        Self {
            data: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Default for InMemoryRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ItemRepository for InMemoryRepository {
    async fn create(&self, request: CreateItemRequest) -> DatabaseResult<Item> {
        let mut items = self.data.write().map_err(|_| DatabaseError::LockError)?;

        let id = Uuid::new_v4().to_string();
        let now = Utc::now();

        let item = Item {
            id: id.clone(),
            name: request.name,
            description: request.description,
            created_at: now,
            updated_at: now,
        };

        items.insert(id, item.clone());
        Ok(item)
    }

    async fn get(&self, id: &str) -> DatabaseResult<Item> {
        let items = self.data.read().map_err(|_| DatabaseError::LockError)?;
        items.get(id).cloned().ok_or(DatabaseError::NotFound)
    }

    async fn update(&self, id: &str, request: UpdateItemRequest) -> DatabaseResult<Item> {
        let mut items = self.data.write().map_err(|_| DatabaseError::LockError)?;

        let item = items.get_mut(id).ok_or(DatabaseError::NotFound)?;

        if let Some(name) = request.name {
            item.name = name;
        }
        if request.description.is_some() {
            item.description = request.description;
        }
        item.updated_at = Utc::now();

        Ok(item.clone())
    }

    async fn delete(&self, id: &str) -> DatabaseResult<()> {
        let mut items = self.data.write().map_err(|_| DatabaseError::LockError)?;
        items.remove(id).ok_or(DatabaseError::NotFound)?;
        Ok(())
    }

    async fn list(&self, limit: usize, offset: usize) -> DatabaseResult<Vec<Item>> {
        let items = self.data.read().map_err(|_| DatabaseError::LockError)?;

        let mut all_items: Vec<Item> = items.values().cloned().collect();
        // Sort by created_at for consistent ordering
        all_items.sort_by(|a, b| a.created_at.cmp(&b.created_at));

        Ok(all_items.into_iter().skip(offset).take(limit).collect())
    }

    async fn count(&self) -> DatabaseResult<usize> {
        let items = self.data.read().map_err(|_| DatabaseError::LockError)?;
        Ok(items.len())
    }

    async fn health_check(&self) -> DatabaseResult<()> {
        // In-memory database is always healthy
        Ok(())
    }
}

/// Future implementation for Convex database
pub struct ConvexRepository {
    #[allow(dead_code)]
    deployment_url: String,
}

impl ConvexRepository {
    pub fn new(deployment_url: String) -> Self {
        Self { deployment_url }
    }
}

#[async_trait]
impl ItemRepository for ConvexRepository {
    async fn create(&self, _request: CreateItemRequest) -> DatabaseResult<Item> {
        Err(DatabaseError::QueryError("Convex not implemented yet".to_string()))
    }

    async fn get(&self, _id: &str) -> DatabaseResult<Item> {
        Err(DatabaseError::QueryError("Convex not implemented yet".to_string()))
    }

    async fn update(&self, _id: &str, _request: UpdateItemRequest) -> DatabaseResult<Item> {
        Err(DatabaseError::QueryError("Convex not implemented yet".to_string()))
    }

    async fn delete(&self, _id: &str) -> DatabaseResult<()> {
        Err(DatabaseError::QueryError("Convex not implemented yet".to_string()))
    }

    async fn list(&self, _limit: usize, _offset: usize) -> DatabaseResult<Vec<Item>> {
        Err(DatabaseError::QueryError("Convex not implemented yet".to_string()))
    }

    async fn count(&self) -> DatabaseResult<usize> {
        Err(DatabaseError::QueryError("Convex not implemented yet".to_string()))
    }

    async fn health_check(&self) -> DatabaseResult<()> {
        // TODO: Implement actual health check
        Ok(())
    }
}

/// Metrics wrapper for ItemRepository
pub struct MetricsRepository {
    inner: Arc<dyn ItemRepository>,
}

impl MetricsRepository {
    pub fn new(inner: Arc<dyn ItemRepository>) -> Self {
        DATABASE_CONNECTIONS.inc();
        Self { inner }
    }
}

impl Drop for MetricsRepository {
    fn drop(&mut self) {
        DATABASE_CONNECTIONS.dec();
    }
}

#[async_trait]
impl ItemRepository for MetricsRepository {
    async fn create(&self, request: CreateItemRequest) -> DatabaseResult<Item> {
        let timer = Timer::new();
        let result = self.inner.create(request).await;
        track_database_query("create", "items", result.is_ok(), timer.elapsed_seconds());

        if result.is_ok() {
            track_item_created();
        }

        result
    }

    async fn get(&self, id: &str) -> DatabaseResult<Item> {
        let timer = Timer::new();
        let result = self.inner.get(id).await;
        track_database_query("get", "items", result.is_ok(), timer.elapsed_seconds());
        result
    }

    async fn update(&self, id: &str, request: UpdateItemRequest) -> DatabaseResult<Item> {
        let timer = Timer::new();
        let result = self.inner.update(id, request).await;
        track_database_query("update", "items", result.is_ok(), timer.elapsed_seconds());

        if result.is_ok() {
            track_item_updated();
        }

        result
    }

    async fn delete(&self, id: &str) -> DatabaseResult<()> {
        let timer = Timer::new();
        let result = self.inner.delete(id).await;
        track_database_query("delete", "items", result.is_ok(), timer.elapsed_seconds());

        if result.is_ok() {
            track_item_deleted();
        }

        result
    }

    async fn list(&self, limit: usize, offset: usize) -> DatabaseResult<Vec<Item>> {
        let timer = Timer::new();
        let result = self.inner.list(limit, offset).await;
        track_database_query("list", "items", result.is_ok(), timer.elapsed_seconds());
        result
    }

    async fn count(&self) -> DatabaseResult<usize> {
        let timer = Timer::new();
        let result = self.inner.count().await;
        track_database_query("count", "items", result.is_ok(), timer.elapsed_seconds());
        result
    }

    async fn health_check(&self) -> DatabaseResult<()> {
        let timer = Timer::new();
        let result = self.inner.health_check().await;
        track_database_query("health_check", "database", result.is_ok(), timer.elapsed_seconds());
        result
    }
}

/// Factory function to create the appropriate repository based on config
pub fn create_repository(config: &Config) -> Arc<dyn ItemRepository> {
    let base_repo: Arc<dyn ItemRepository> = match config.database.db_type.as_str() {
        "memory" => Arc::new(InMemoryRepository::new()),
        "convex" => {
            let url = config
                .database
                .convex_deployment_url
                .as_ref()
                .expect("Convex deployment URL required");
            Arc::new(ConvexRepository::new(url.clone()))
        }
        _ => panic!("Unknown database type: {}", config.database.db_type),
    };

    // Wrap with metrics tracking
    Arc::new(MetricsRepository::new(base_repo))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_in_memory_crud_operations() {
        let repo = InMemoryRepository::new();

        // Create
        let create_req = CreateItemRequest {
            name: "Test Item".to_string(),
            description: Some("Test Description".to_string()),
        };
        let created = repo.create(create_req).await.unwrap();
        assert_eq!(created.name, "Test Item");

        // Get
        let retrieved = repo.get(&created.id).await.unwrap();
        assert_eq!(retrieved.id, created.id);

        // Update
        let update_req = UpdateItemRequest {
            name: Some("Updated Name".to_string()),
            description: None,
        };
        let updated = repo.update(&created.id, update_req).await.unwrap();
        assert_eq!(updated.name, "Updated Name");
        assert_eq!(updated.description, Some("Test Description".to_string()));

        // List
        let items = repo.list(10, 0).await.unwrap();
        assert_eq!(items.len(), 1);

        // Count
        let count = repo.count().await.unwrap();
        assert_eq!(count, 1);

        // Delete
        repo.delete(&created.id).await.unwrap();
        let result = repo.get(&created.id).await;
        assert!(matches!(result, Err(DatabaseError::NotFound)));
    }

    #[tokio::test]
    async fn test_health_check() {
        let repo = InMemoryRepository::new();
        assert!(repo.health_check().await.is_ok());
    }
}
