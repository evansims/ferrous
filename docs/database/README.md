# Database Architecture

Estuary uses a flexible database abstraction layer that allows switching between different database implementations without changing application code.

## Overview

The database layer is built around Rust traits, providing a clean interface between the application logic and the underlying storage mechanism.

### Core Components

1. **Database Trait** (`src/database/mod.rs`)
   - Main interface for database operations
   - Provides access to repositories
   - Handles health checks

2. **Repository Pattern** (`src/database/repositories/`)
   - Separate interfaces for each domain entity
   - Currently implements `ItemRepository`
   - Async operations using `async-trait`

3. **Implementations** (`src/database/implementations/`)
   - Concrete database implementations
   - Currently supports:
     - In-memory storage
     - [Convex](./convex.md)

## Database Trait

```rust
#[async_trait]
pub trait Database: Send + Sync {
    fn items(&self) -> Arc<dyn ItemRepository>;
    async fn health_check(&self) -> Result<(), DatabaseError>;
}
```

## Repository Pattern

Each domain entity has its own repository trait:

```rust
#[async_trait]
pub trait ItemRepository: Send + Sync {
    async fn list(&self, limit: usize, offset: usize) -> DatabaseResult<Vec<Item>>;
    async fn count(&self) -> DatabaseResult<usize>;
    async fn get(&self, id: &str) -> DatabaseResult<Item>;
    async fn create(&self, request: CreateItemRequest) -> DatabaseResult<Item>;
    async fn update(&self, id: &str, request: UpdateItemRequest) -> DatabaseResult<Item>;
    async fn delete(&self, id: &str) -> DatabaseResult<()>;
}
```

## Configuration

Database selection is controlled via environment variables:

```env
# Options: memory (default), convex
DATABASE_TYPE=memory

# Database-specific configuration
CONVEX_DEPLOYMENT_URL=https://your-project.convex.cloud
```

## Error Handling

The database layer defines its own error types:

```rust
#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("Connection error: {0}")]
    ConnectionError(String),
    
    #[error("Query error: {0}")]
    QueryError(String),
    
    #[error("Item not found")]
    NotFound,
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
}
```

## Adding a New Database Implementation

To add a new database implementation:

1. Create a new module in `src/database/implementations/`
2. Implement the `Database` trait
3. Implement all required repository traits
4. Update `DatabaseFactory::create()` to handle the new database type
5. Add configuration options to `.env.example`
6. Document the implementation in this folder

## Best Practices

1. **Consistency**: All database operations should return `DatabaseResult<T>`
2. **Error Handling**: Map database-specific errors to `DatabaseError` variants
3. **Async Safety**: Ensure all implementations are `Send + Sync`
4. **Testing**: Each implementation should have integration tests
5. **Documentation**: Document any database-specific setup requirements

## Current Implementations

- [In-Memory Database](./in-memory.md) - Simple HashMap-based storage for development
- [Convex Database](./convex.md) - Serverless database with real-time sync

## Future Implementations

- **Redis** - For caching and session storage
- **DynamoDB** - AWS serverless database option