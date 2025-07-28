# Estuary

A minimal, reliable REST API service built with Rust and Axum.

## Features

- Clean, minimal architecture following Rust best practices
- RESTful API with CRUD operations for demo items
- Built-in error handling and fault tolerance
- Request/response tracing with structured logging
- CORS support for cross-origin requests
- Health check endpoint for monitoring

## Running the Service

### Using Make commands
```bash
# Build for production
make build

# Run in development mode
make run

# Run with hot-reload (auto-restart on file changes)
make watch

# Show all available commands
make help
```

### Using Cargo directly
```bash
# Build the project
cargo build

# Run in development mode
cargo run

# Run with auto-reload (watches for file changes)
cargo watch -x run

# Run with auto-reload and clear screen on restart
cargo watch -c -x run

# Run in release mode for production
cargo build --release
./target/release/estuary
```

The service will start on `http://0.0.0.0:3000`

## API Endpoints

### Health Check
- `GET /` - Returns service health status

### Items Resource
- `GET /api/v1/items` - List all items (supports pagination with `?limit=20&offset=0`)
- `POST /api/v1/items` - Create a new item
- `GET /api/v1/items/{id}` - Get a specific item
- `PUT /api/v1/items/{id}` - Update an item
- `DELETE /api/v1/items/{id}` - Delete an item

## Example Requests

### Create an item
```bash
curl -X POST http://localhost:3000/api/v1/items \
  -H "Content-Type: application/json" \
  -d '{"name": "Test Item", "description": "A test item"}'
```

### List items
```bash
curl http://localhost:3000/api/v1/items
```

### Get specific item
```bash
curl http://localhost:3000/api/v1/items/{item-id}
```

### Update an item
```bash
curl -X PUT http://localhost:3000/api/v1/items/{item-id} \
  -H "Content-Type: application/json" \
  -d '{"name": "Updated Name"}'
```

### Delete an item
```bash
curl -X DELETE http://localhost:3000/api/v1/items/{item-id}
```

## Environment Variables

The application supports loading environment variables from a `.env` file. Copy `.env.example` to `.env` and customize as needed.

- `PORT` - Server port (default: `3000`)
- `RUST_LOG` - Control logging levels (default: `estuary=debug,tower_http=debug`)

## Architecture Notes

- Uses `RwLock` for thread-safe in-memory storage (replace with a real database for production)
- Implements proper error handling with typed responses
- Follows RESTful conventions with appropriate HTTP status codes
- Minimal dependencies for reliability and security

## Maintenance

This repository uses Dependabot to keep dependencies up to date:
- Checks for Cargo dependency updates weekly
- Automatically creates PRs for updates
- Groups minor and patch updates to reduce noise
- Runs CI checks before merging