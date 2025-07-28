# Getting Started

This guide will help you get Estuary up and running on your local machine.

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (1.75 or later)
- [Git](https://git-scm.com/downloads)
- (Optional) [cargo-watch](https://github.com/watchexec/cargo-watch) for hot-reload

## Installation

### 1. Clone the Repository

```bash
git clone <repository-url>
cd estuary
```

### 2. Install Dependencies

```bash
# Install cargo-watch for development (optional but recommended)
cargo install cargo-watch

# Build the project
cargo build
```

### 3. Configure Environment

```bash
# Copy the example environment file
cp .env.example .env

# Edit .env to configure your settings
# For development, the defaults should work fine
```

## Running the Application

### Development Mode

```bash
# Run with hot-reload (recommended)
make watch

# Or run normally
make run

# Or use cargo directly
cargo run
```

### Production Build

```bash
# Build optimized binary
make build

# Run the release build
./target/release/estuary
```

## Configuration Options

### Basic Configuration

Edit your `.env` file to configure the application:

```env
# Server port (default: 3000)
PORT=3000

# Logging level
RUST_LOG=estuary=debug,tower_http=debug

# Database type (memory, convex)
DATABASE_TYPE=memory
```

### Database Configuration

#### In-Memory (Default)

No additional configuration needed. Data is stored in RAM and lost on restart.

```env
DATABASE_TYPE=memory
```

#### Convex

Requires a Convex deployment:

```env
DATABASE_TYPE=convex
CONVEX_DEPLOYMENT_URL=https://your-project.convex.cloud
```

See [Convex setup guide](./database/convex.md) for detailed instructions.

## Testing the API

Once the server is running, you can test it using curl:

### Health Check

```bash
curl http://localhost:3000/
```

### Create an Item

```bash
curl -X POST http://localhost:3000/api/v1/items \
  -H "Content-Type: application/json" \
  -d '{"name": "My First Item", "description": "Testing Estuary"}'
```

### List Items

```bash
curl http://localhost:3000/api/v1/items
```

### Get a Specific Item

```bash
curl http://localhost:3000/api/v1/items/{id}
```

### Update an Item

```bash
curl -X PUT http://localhost:3000/api/v1/items/{id} \
  -H "Content-Type: application/json" \
  -d '{"name": "Updated Name"}'
```

### Delete an Item

```bash
curl -X DELETE http://localhost:3000/api/v1/items/{id}
```

## Development Workflow

### 1. Make Changes

Edit the source files in the `src/` directory.

### 2. Auto-Reload

If using `make watch`, the server will automatically restart when you save changes.

### 3. Check Logs

The application uses structured logging. Adjust the log level in `.env`:

```env
# Options: error, warn, info, debug, trace
RUST_LOG=estuary=debug
```

### 4. Run Tests

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_name
```

### 5. Format Code

```bash
# Format all code
cargo fmt

# Check formatting without changes
cargo fmt -- --check
```

### 6. Lint Code

```bash
# Run clippy linter
cargo clippy

# Run with all targets
cargo clippy --all-targets --all-features
```

## Project Structure

```
src/
├── main.rs              # Entry point
├── lib.rs               # Library root
├── routes.rs            # Route definitions
├── handlers/            # Request handlers
│   └── items.rs        # Items endpoints
├── models/              # Data models
│   └── item.rs         # Item model
├── database/            # Database layer
│   ├── mod.rs          # Database traits
│   ├── repositories/   # Repository interfaces
│   └── implementations/ # Database implementations
├── middleware/          # HTTP middleware
├── error.rs            # Error handling
└── state.rs            # Application state
```

## Next Steps

- Read the [API Reference](./api-reference.md) for endpoint details
- Learn about the [Database Architecture](./database/README.md)
- Set up a [Convex database](./database/convex.md) for persistence
- Configure [GitHub Actions](./.github/workflows/) for CI/CD

## Troubleshooting

### Port Already in Use

If you get a "port already in use" error:

```bash
# Find the process using port 3000
lsof -i :3000

# Or change the port in .env
PORT=3001
```

### Compilation Errors

```bash
# Clean build artifacts
cargo clean

# Update dependencies
cargo update

# Rebuild
cargo build
```

### Database Connection Issues

Check your `.env` configuration and ensure any external databases are accessible.

## Getting Help

- Check the [documentation](./README.md)
- Review the [API reference](./api-reference.md)
- Search existing [issues](https://github.com/your-repo/issues)
- Open a new issue with details about your problem