# Ferrous Documentation

Welcome to the Ferrous project documentation. Ferrous is a Rust-based web service designed to serve REST APIs with ultra-reliability, fault-tolerance, and resilience.

## Table of Contents

- [Getting Started](./getting-started.md)
- [Architecture](./architecture.md)
- [API Reference](./api-reference.md)
- [API Versioning](./api-versioning.md)
- [Authentication](./authentication.md)
- [Database](./database/)
  - [Overview](./database/README.md)
  - [Convex Implementation](./database/convex.md)
- [Development](./development.md)
- [Deployment](./deployment.md)

## Quick Start

```bash
# Clone the repository
git clone <repository-url>
cd ferrous

# Copy environment configuration
cp .env.example .env

# Build the project
make build

# Run the development server
make run

# Run with auto-reload
make watch
```

## Project Structure

```
ferrous/
├── src/
│   ├── main.rs              # Application entry point
│   ├── lib.rs               # Library root
│   ├── config.rs            # Configuration management
│   ├── db.rs                # Database abstraction and implementations
│   ├── error.rs             # Error handling
│   ├── handlers.rs          # HTTP request handlers
│   ├── metrics.rs           # Prometheus metrics
│   ├── middleware/          # HTTP middleware
│   │   ├── auth.rs         # JWT authentication
│   │   ├── rate_limit.rs   # Rate limiting
│   │   ├── security.rs     # Security headers
│   │   └── ...             # Other middleware
│   ├── models.rs            # Data models and DTOs
│   ├── openapi.rs           # OpenAPI documentation
│   ├── routes.rs            # API route definitions
│   ├── state.rs             # Application state
│   └── validation.rs        # Input validation
├── convex/                  # Convex database functions
├── docs/                    # Documentation
├── monitoring/              # Prometheus and Grafana configs
├── .github/                 # GitHub workflows
├── docker-compose.yml       # Base Docker configuration
├── docker-compose.*.yml     # Environment-specific overrides
├── Dockerfile              # Container image definition
├── Cargo.toml              # Rust dependencies
├── Makefile                # Build commands
└── .env.example            # Environment configuration template
```

## Key Features

### Core Architecture
- **Modular Architecture**: Clean separation of concerns with a modular codebase structure
- **Database Abstraction**: Pluggable database layer supporting multiple implementations
- **Environment-based Configuration**: Easy configuration through environment variables
- **Hot Reload**: Development server with automatic recompilation on changes

### API Features
- **OpenAPI Documentation**: Machine-readable API specification at `/openapi.json`
- **Structured Error Responses**: Consistent error format with request correlation
- **Input Validation**: Comprehensive validation with field length limits and sanitization
- **Request ID Tracking**: Unique request IDs for debugging and log correlation

### Security & Reliability
- **JWT Authentication**: Token-based authentication with configurable secret
- **Rate Limiting**: Configurable per-IP rate limits with informative headers
- **Security Headers**: Comprehensive security headers (CSP, HSTS, etc.)
- **Graceful Shutdown**: Proper connection draining and shutdown handling

### Observability
- **Health Endpoints**: Multiple health check endpoints for different purposes
- **System Metrics**: Memory usage, uptime, and database health monitoring
- **Structured Logging**: Correlation IDs and detailed request/response logging
- **Production Ready**: Built with reliability and fault-tolerance in mind

## Technology Stack

- **Language**: Rust
- **Web Framework**: Axum 0.8
- **Async Runtime**: Tokio 1.47
- **Database Options**: In-memory, Convex (more coming soon)
- **Serialization**: Serde
- **HTTP Middleware**: Tower, Tower-HTTP

## Contributing

See [CONTRIBUTING.md](../CONTRIBUTING.md) for guidelines on contributing to this project.

## License

This project is licensed under the terms specified in the [LICENSE](../LICENSE) file.