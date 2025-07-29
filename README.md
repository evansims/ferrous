# Ferrous

This template provides a starting point for building your own REST API service using Rust and Axum. It's intended to provide a starting point for building your own APIs, using clean architecture and modular design following Rust best practices. Fork it and build something cool!

## Documentation

For comprehensive documentation, see the [docs](./docs/) directory:

- [Getting Started](./docs/getting-started.md) - Installation and setup guide
- [API Reference](./docs/api-reference.md) - Complete API endpoint documentation
- [API Versioning](./docs/api-versioning.md) - Version negotiation and deprecation strategy
- [Authentication Guide](./docs/authentication.md) - JWT/JWKS authentication setup
- [Database Documentation](./docs/database/) - Database architecture and implementations
- [Architecture Overview](./docs/README.md) - Project structure and design decisions

## Quick Start

```bash
# Clone and enter the project
git clone <repository-url>
cd ferrous

# Copy environment configuration
cp .env.example .env

# Run the development server
make run
```

The service will start on `http://localhost:3000`

## Features

### Core Functionality
- **Clean Architecture** - Modular design following Rust best practices
- **Database Abstraction** - Pluggable database layer with multiple implementations
- **RESTful API** - Full CRUD operations with proper HTTP semantics
- **Comprehensive Test Suite** - Unit tests, integration tests, and test utilities

### API Features
- **OpenAPI Documentation** - Machine-readable API spec at `/openapi.json`
- **Structured Error Responses** - Consistent error format with machine-readable codes
- **Input Validation** - Field length limits (name: 1-255 chars, description: max 1000 chars) with sanitization
- **Request ID Tracking** - Unique request IDs for debugging and log correlation

### Security & Reliability
- **JWKS Authentication** - JWT validation with support for multiple JWKS endpoints (optional in dev)
- **Rate Limiting** - Configurable per-IP rate limits (default: 1000 req/min)
- **Security Headers** - CSP, X-Frame-Options, HSTS, and more
- **Graceful Shutdown** - Proper connection draining with configurable timeout

### Observability
- **Enhanced Health Checks** - Comprehensive health endpoint with system metrics
- **Multiple Health Endpoints** - `/health` (detailed), `/health/live`, `/health/ready`
- **Structured Logging** - Request/response logging with correlation IDs
- **Performance Monitoring** - Memory usage, uptime, and database health tracking

## API Endpoints

### Documentation
- `GET /openapi.json` - OpenAPI 3.0 specification

### Health Checks
- `GET /` - Basic health status
- `GET /health` - Comprehensive health check with system metrics
- `GET /health/live` - Liveness probe for container orchestration
- `GET /health/ready` - Readiness probe with database connectivity check

### Monitoring
- `GET /metrics` - Prometheus metrics endpoint

### Items Resource
- `GET /api/v1/items` - List all items (supports pagination)
- `POST /api/v1/items` - Create a new item
- `GET /api/v1/items/{id}` - Get a specific item
- `PUT /api/v1/items/{id}` - Update an item
- `DELETE /api/v1/items/{id}` - Delete an item

See the [API Reference](./docs/api-reference.md) for detailed endpoint documentation.

## Database Support

Ferrous supports multiple database backends through its abstraction layer:

- **In-Memory** (default) - Fast development and testing
- **Convex** - Serverless database with real-time sync

Configure via the `DATABASE_TYPE` environment variable. See [Database Documentation](./docs/database/) for details.

## Development

### Using Make commands
```bash
make build    # Build for production
make run      # Run in development mode
make watch    # Run with hot-reload
make help     # Show all available commands
```

### Using Cargo directly
```bash
cargo build              # Build the project
cargo run                # Run in development mode
cargo watch -c -x run    # Run with auto-reload
cargo test               # Run tests
```

## Project Structure

```
ferrous/
├── src/                    # Source code
│   ├── db.rs               # Database abstraction and implementations
│   ├── handlers.rs         # HTTP request handlers
│   ├── middleware/         # HTTP middleware (auth, rate limit, etc.)
│   ├── models.rs           # Data models and DTOs
│   ├── config.rs           # Configuration management
│   ├── error.rs            # Error types and handling
│   ├── routes.rs           # API route definitions
│   └── main.rs             # Application entry point
├── docs/                   # Documentation
├── convex/                 # Convex database functions
├── monitoring/             # Prometheus and Grafana configs
└── .github/                # CI/CD workflows
```

## Maintenance

This repository uses automated dependency management:
- Dependabot checks for updates weekly
- CI/CD pipeline validates all changes
- Comprehensive test coverage ensures reliability

## License

See [LICENSE](./LICENSE) file for details.
