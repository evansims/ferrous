pub mod in_memory;

use crate::database::{Database, DatabaseError};
use std::sync::Arc;

/// Factory for creating database instances based on configuration
pub struct DatabaseFactory;

impl DatabaseFactory {
    /// Create a database instance based on the DATABASE_TYPE environment variable
    pub async fn create() -> Result<Arc<dyn Database>, DatabaseError> {
        let db_type = std::env::var("DATABASE_TYPE").unwrap_or_else(|_| "memory".to_string());
        
        match db_type.as_str() {
            "memory" | "in-memory" => Ok(Arc::new(in_memory::InMemoryDatabase::new())),
            // Future implementations can be added here:
            // "postgres" => Ok(Arc::new(postgres::PostgresDatabase::new().await?)),
            // "sqlite" => Ok(Arc::new(sqlite::SqliteDatabase::new().await?)),
            _ => Err(DatabaseError::ConnectionError(format!(
                "Unknown database type: {}",
                db_type
            ))),
        }
    }
}