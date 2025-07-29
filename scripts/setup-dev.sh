#!/bin/bash
# Development environment setup script

set -e

echo "Setting up development environment for Ferrous..."

# Configure git to use our hooks
echo "Configuring git hooks..."
git config core.hooksPath .githooks

# Install required tools
echo "Checking required tools..."

# Check if cargo-watch is installed
if ! command -v cargo-watch &> /dev/null; then
    echo "Installing cargo-watch..."
    cargo install cargo-watch
fi

# Check if cargo-audit is installed
if ! command -v cargo-audit &> /dev/null; then
    echo "Installing cargo-audit..."
    cargo install cargo-audit
fi

# Ensure rust-toolchain.toml is respected
echo "Rust toolchain: $(rustc --version)"

echo "✅ Development environment setup complete!"
echo ""
echo "Tips:"
echo "- Run 'make ci-local' before pushing to catch CI issues early"
echo "- Git hooks are now active and will run checks before commits"
echo "- The rust-toolchain.toml file ensures everyone uses the same Rust version"