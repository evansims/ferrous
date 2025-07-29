pub mod implementations;
pub mod repositories;

use async_trait::async_trait;
use std::sync::Arc;

pub use implementations::DatabaseFactory;
pub use repositories::*;

/// Main database trait that all database implementations must implement
#[async_trait]
pub trait Database: Send + Sync {
    /// Get the items repository
    fn items(&self) -> Arc<dyn ItemRepository>;

    /// Health check for the database connection
    async fn health_check(&self) -> Result<(), DatabaseError>;
}

/// Database errors that can occur across all implementations
#[derive(Debug, thiserror::Error)]
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
