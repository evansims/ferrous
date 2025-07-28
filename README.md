# Estuary

A minimal, reliable REST API service built with Rust and Axum.

## Documentation

For comprehensive documentation, see the [docs](./docs/) directory:

- [Getting Started](./docs/getting-started.md) - Installation and setup guide
- [API Reference](./docs/api-reference.md) - Complete API endpoint documentation
- [Database Documentation](./docs/database/) - Database architecture and implementations
- [Architecture Overview](./docs/README.md) - Project structure and design decisions

## Quick Start

```bash
# Clone and enter the project
git clone <repository-url>
cd estuary

# Copy environment configuration
cp .env.example .env

# Run the development server
make run
```

The service will start on `http://localhost:3000`

## Features

- **Clean Architecture** - Modular design following Rust best practices
- **Database Abstraction** - Pluggable database layer with multiple implementations
- **RESTful API** - Full CRUD operations with proper HTTP semantics
- **Production Ready** - Error handling, structured logging, health checks
- **Developer Experience** - Hot reload, comprehensive docs, simple setup
- **Fault Tolerance** - Built for reliability and resilience

## API Endpoints

### Health Check
- `GET /` - Returns service health status

### Items Resource
- `GET /api/v1/items` - List all items (supports pagination)
- `POST /api/v1/items` - Create a new item
- `GET /api/v1/items/{id}` - Get a specific item
- `PUT /api/v1/items/{id}` - Update an item
- `DELETE /api/v1/items/{id}` - Delete an item

See the [API Reference](./docs/api-reference.md) for detailed endpoint documentation.

## Database Support

Estuary supports multiple database backends through its abstraction layer:

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
estuary/
├── src/                     # Source code
│   ├── database/           # Database abstraction layer
│   ├── handlers/           # Request handlers
│   ├── middleware/         # HTTP middleware
│   └── models/             # Data models
├── docs/                    # Documentation
├── convex/                  # Convex function examples
└── .github/                 # CI/CD workflows
```

## Maintenance

This repository uses automated dependency management:
- Dependabot checks for updates weekly
- CI/CD pipeline validates all changes
- Comprehensive test coverage ensures reliability

## License

See [LICENSE](./LICENSE) file for details.