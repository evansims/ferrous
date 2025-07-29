pub mod convex;
pub mod in_memory;

use crate::{
    config::DatabaseConfig,
    database::{Database, DatabaseError, MetricsDatabase},
};
use std::sync::Arc;

/// Factory for creating database instances based on configuration
pub struct DatabaseFactory;

impl DatabaseFactory {
    /// Create a database instance based on configuration
    pub async fn create() -> Result<Arc<dyn Database>, DatabaseError> {
        // For backward compatibility, try to load from env vars if no config is passed
        let db_type = std::env::var("DATABASE_TYPE").unwrap_or_else(|_| "memory".to_string());
        let convex_url = std::env::var("CONVEX_DEPLOYMENT_URL").ok();

        let config = DatabaseConfig {
            db_type: db_type.clone(),
            convex_deployment_url: convex_url,
        };

        Self::create_with_config(&config).await
    }

    /// Create a database instance with explicit configuration
    pub async fn create_with_config(
        config: &DatabaseConfig,
    ) -> Result<Arc<dyn Database>, DatabaseError> {
        let base_db: Arc<dyn Database> = match config.db_type.as_str() {
            "memory" | "in-memory" => Arc::new(in_memory::InMemoryDatabase::new()),
            "convex" => {
                let deployment_url = config.convex_deployment_url.as_ref().ok_or_else(|| {
                    DatabaseError::ConnectionError(
                        "CONVEX_DEPLOYMENT_URL is required for Convex database".to_string(),
                    )
                })?;
                Arc::new(convex::ConvexDatabase::new(deployment_url).await?)
            }
            // Future implementations can be added here
            _ => {
                return Err(DatabaseError::ConnectionError(format!(
                    "Unknown database type: {}",
                    config.db_type
                )))
            }
        };

        // Wrap with metrics tracking
        Ok(Arc::new(MetricsDatabase::new(base_db)))
    }
}
