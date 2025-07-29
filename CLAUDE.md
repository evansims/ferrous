# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Estuary is a minimal REST API service built with Rust and Axum, designed for reliability and clean architecture.

## Development Commands

### Using Make (Recommended)
```bash
# Primary commands
make build   # Build in release mode
make run     # Run in development mode
make watch   # Run with hot-reload (clears screen on restart)

# Additional commands
make clean   # Clean build artifacts
make check   # Check compilation without building
make test    # Run tests
make fmt     # Format code
make lint    # Run clippy linter
make ci      # Run CI checks (format check + lint with warnings as errors)
make help    # Show all available commands
```

### Using Cargo Directly
```bash
# Build the project
cargo build

# Run in development mode with debug logging
cargo run

# Run with hot-reload (watches for file changes and restarts automatically)
cargo watch -x run

# Run with hot-reload and clear screen on each restart
cargo watch -c -x run

# Run in release mode (optimized)
cargo build --release
./target/release/estuary

# Check for compilation errors without building
cargo check

# Format code
cargo fmt

# Run linter
cargo clippy

# Clean build artifacts
cargo clean
```

## Architecture

The service follows a modular architecture with clear separation of concerns:

### Module Structure
- `src/main.rs` - Application entry point, server initialization
- `src/lib.rs` - Library root exposing public modules
- `src/config.rs` - Simplified configuration using environment variables
- `src/db.rs` - Database abstraction with repository pattern and metrics
- `src/error.rs` - Centralized error handling with `AppError` enum
- `src/handlers.rs` - All HTTP handlers consolidated in one file
- `src/metrics.rs` - Prometheus metrics collection
- `src/middleware/` - Middleware implementations
  - `mod.rs` - Middleware composition
  - `security.rs` - CORS and security headers
  - `observability.rs` - Request tracing and metrics
  - `auth.rs` - JWT authentication
  - `rate_limit.rs` - Rate limiting
  - `version.rs` - API versioning
- `src/models.rs` - Domain models (Item, CreateItemRequest, UpdateItemRequest)
- `src/openapi.rs` - OpenAPI documentation
- `src/routes.rs` - Route configuration
- `src/state.rs` - Application state management
- `src/validation.rs` - Request validation

### Key Design Patterns

1. **Database Abstraction**:
   - Repository pattern with `ItemRepository` trait
   - Swappable implementations via `create_repository()` factory
   - Currently supports in-memory storage, easily extensible
   - Metrics tracking built into repository wrapper

2. **State Management**: `SharedState` holds `Arc<dyn ItemRepository>` for database access

3. **Error Handling**:
   - Centralized through `AppError` enum that implements `IntoResponse`
   - Database errors automatically map to appropriate HTTP status codes
   - All handlers return `AppResult<T>`

4. **Middleware Stack**: Applied in `middleware::add_middleware()`:
   - TraceLayer (outermost) - HTTP request/response logging
   - CorsLayer - Enables cross-origin requests

5. **Type Organization**:
   - Models: `Item`, `CreateItemRequest`, `UpdateItemRequest` in `models.rs`
   - Repository: `ItemRepository` trait and implementations in `db.rs`
   - Handler types: `ListQuery`, `ListResponse` in `handlers.rs`

## Important Implementation Details

- The server binds to `0.0.0.0:3000` by default
- Health check endpoint is at the root path `/`
- Pagination defaults: limit=20 (max 100), offset=0
- All timestamps use `chrono::DateTime<chrono::Utc>`
- IDs are generated using `uuid::Uuid::new_v4()`
- Update operations preserve fields not provided in the request
- DELETE operations return 204 No Content on success

## Environment Variables

The application loads environment variables from `.env` file if present (using dotenvy).

- `PORT`: Server port (default: `3000`)
- `RUST_LOG`: Controls logging verbosity (default: `estuary=debug,tower_http=debug`)

See `.env.example` for additional configuration options that can be added in the future.

## Dependencies & Compatibility

- Tokio 1.47 - Latest async runtime with improved performance and scheduling
- Axum 0.8.x - Uses new path parameter syntax `/{param}` instead of `/:param`
- Tower-HTTP 0.6.x - Compatible with Axum 0.8
- When adding middleware, ensure correct layer ordering to avoid trait bound errors
- Minimum Rust version: 1.70 (required by Tokio 1.47)

## Dependency Management

The repository uses Dependabot for automated dependency updates:
- Configuration: `.github/dependabot.yml`
- Weekly checks on Mondays at 09:00 UTC
- Groups minor/patch updates to reduce PR noise
- Auto-merge workflow for patch and minor updates
- CI pipeline runs on all PRs to ensure compatibility
