# In-Memory Database Implementation

The in-memory database implementation provides a simple, fast storage solution ideal for development and testing.

## Overview

The in-memory implementation uses Rust's `HashMap` wrapped in `Arc<RwLock<>>` to provide thread-safe access to data stored entirely in RAM.

## Features

- **Zero Configuration**: Works out of the box with no setup required
- **Fast Performance**: All operations happen in memory
- **Thread-Safe**: Uses `RwLock` for concurrent read/write access
- **UUID Generation**: Automatically generates unique IDs for new items
- **Timestamps**: Tracks creation and update times using `chrono`

## Configuration

To use the in-memory database, set the following in your `.env` file:

```env
DATABASE_TYPE=memory
```

Or simply omit the `DATABASE_TYPE` variable, as `memory` is the default.

## Implementation Details

### Data Structure

```rust
pub struct InMemoryDatabase {
    items: Arc<InMemoryItemRepository>,
}

pub struct InMemoryItemRepository {
    store: Arc<RwLock<HashMap<String, Item>>>,
}
```

### Operations

All operations are implemented with proper locking:

- **Reads**: Use read locks allowing multiple concurrent readers
- **Writes**: Use write locks ensuring exclusive access during modifications

### Limitations

1. **Data Persistence**: All data is lost when the application stops
2. **Memory Usage**: All data must fit in available RAM
3. **No Query Optimization**: Simple linear scans for filtering/counting
4. **Basic Pagination**: Offset-based pagination without optimization

## Use Cases

The in-memory database is ideal for:

- **Development**: Quick iteration without database setup
- **Testing**: Unit and integration tests with predictable state
- **Prototyping**: Rapid prototyping of new features
- **Demo Environments**: Showcasing functionality without infrastructure

## Example Usage

```bash
# Start the server with in-memory database
DATABASE_TYPE=memory cargo run

# Create an item
curl -X POST http://localhost:3000/api/v1/items \
  -H "Content-Type: application/json" \
  -d '{"name": "Test Item", "description": "In-memory storage"}'

# List items (data persists until server restart)
curl http://localhost:3000/api/v1/items
```

## Migration to Production Databases

When moving from in-memory to a production database:

1. Change the `DATABASE_TYPE` environment variable
2. Add database-specific configuration (connection strings, etc.)
3. No code changes required thanks to the abstraction layer
4. Consider data migration strategies if needed

## Performance Characteristics

- **Create**: O(1) - HashMap insertion
- **Read by ID**: O(1) - HashMap lookup
- **Update**: O(1) - HashMap access
- **Delete**: O(1) - HashMap removal
- **List**: O(n) - Requires iterating all items
- **Count**: O(n) - Requires counting all items

## Thread Safety

The implementation uses `RwLock` which allows:
- Multiple concurrent readers
- Single writer with exclusive access
- No readers while writing

This provides good performance for read-heavy workloads while maintaining data consistency.