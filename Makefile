.PHONY: build run watch clean check test fmt lint ci audit ci-local help

# Build the project in release mode
build:
	cargo build --release

# Run the project in development mode
run:
	cargo run

# Run with hot-reload (watches for changes)
watch:
	cargo watch -c -x run

# Clean build artifacts
clean:
	cargo clean

# Check for compilation errors
check:
	cargo check

# Run tests
test:
	cargo test

# Format code
fmt:
	cargo fmt

# Run linter
lint:
	cargo clippy

# Run security audit
audit:
	cargo audit

# Run format check and linter (useful for CI)
ci: check
	cargo fmt -- --check
	cargo clippy -- -D warnings

# Run all CI checks locally (exactly as CI does)
ci-local: 
	@echo "Running CI checks with pinned toolchain..."
	@echo "Toolchain: $$(rustc --version)"
	cargo fmt -- --check
	cargo clippy -- -D warnings
	cargo test --verbose
	cargo audit
	@echo "All CI checks passed!"

# Help command
help:
	@echo "Available commands:"
	@echo "  make build  - Build the project in release mode"
	@echo "  make run    - Run the project in development mode"
	@echo "  make watch  - Run with hot-reload (auto-restart on changes)"
	@echo "  make clean  - Clean build artifacts"
	@echo "  make check  - Check for compilation errors"
	@echo "  make test   - Run tests"
	@echo "  make fmt    - Format code"
	@echo "  make lint   - Run clippy linter"
	@echo "  make ci     - Run CI checks (format & lint)"
	@echo "  make audit  - Run security audit"
	@echo "  make ci-local - Run all CI checks locally"
	@echo "  make help   - Show this help message"

# Default target
.DEFAULT_GOAL := help