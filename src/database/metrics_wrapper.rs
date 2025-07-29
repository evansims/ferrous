use crate::{
    database::{Database, DatabaseError, ItemRepository},
    metrics::{track_database_query, Timer, DATABASE_CONNECTIONS},
    models::{CreateItemRequest, Item, UpdateItemRequest},
};
use async_trait::async_trait;
use std::sync::Arc;

/// Wrapper for database that tracks metrics
pub struct MetricsDatabase {
    inner: Arc<dyn Database>,
}

impl MetricsDatabase {
    pub fn new(database: Arc<dyn Database>) -> Self {
        DATABASE_CONNECTIONS.inc();
        Self { inner: database }
    }
}

impl Drop for MetricsDatabase {
    fn drop(&mut self) {
        DATABASE_CONNECTIONS.dec();
    }
}

#[async_trait]
impl Database for MetricsDatabase {
    async fn health_check(&self) -> Result<(), DatabaseError> {
        let timer = Timer::new();
        let result = self.inner.health_check().await;
        track_database_query(
            "health_check",
            "database",
            result.is_ok(),
            timer.elapsed_seconds(),
        );
        result
    }

    fn items(&self) -> Arc<dyn ItemRepository> {
        Arc::new(MetricsItemRepository {
            inner: self.inner.items(),
        })
    }
}

/// Wrapper for item repository that tracks metrics
pub struct MetricsItemRepository {
    inner: Arc<dyn ItemRepository>,
}

#[async_trait]
impl ItemRepository for MetricsItemRepository {
    async fn create(&self, item: CreateItemRequest) -> Result<Item, DatabaseError> {
        let timer = Timer::new();
        let result = self.inner.create(item).await;
        track_database_query("create", "items", result.is_ok(), timer.elapsed_seconds());

        if result.is_ok() {
            crate::metrics::track_item_created();
        }

        result
    }

    async fn get(&self, id: &str) -> Result<Item, DatabaseError> {
        let timer = Timer::new();
        let result = self.inner.get(id).await;
        track_database_query("get", "items", result.is_ok(), timer.elapsed_seconds());
        result
    }

    async fn update(&self, id: &str, item: UpdateItemRequest) -> Result<Item, DatabaseError> {
        let timer = Timer::new();
        let result = self.inner.update(id, item).await;
        track_database_query("update", "items", result.is_ok(), timer.elapsed_seconds());

        if result.is_ok() {
            crate::metrics::track_item_updated();
        }

        result
    }

    async fn delete(&self, id: &str) -> Result<(), DatabaseError> {
        let timer = Timer::new();
        let result = self.inner.delete(id).await;
        track_database_query("delete", "items", result.is_ok(), timer.elapsed_seconds());

        if result.is_ok() {
            crate::metrics::track_item_deleted();
        }

        result
    }

    async fn list(&self, limit: usize, offset: usize) -> Result<Vec<Item>, DatabaseError> {
        let timer = Timer::new();
        let result = self.inner.list(limit, offset).await;
        track_database_query("list", "items", result.is_ok(), timer.elapsed_seconds());
        result
    }

    async fn count(&self) -> Result<usize, DatabaseError> {
        let timer = Timer::new();
        let result = self.inner.count().await;
        track_database_query("count", "items", result.is_ok(), timer.elapsed_seconds());
        result
    }
}
