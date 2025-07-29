# CI/Local Development Parity

This document explains how we ensure consistency between local development and CI environments.

## Key Components

### 1. **rust-toolchain.toml**
Pins the exact Rust version for all developers:
```toml
[toolchain]
channel = "1.86.0"
components = ["rustfmt", "clippy"]
profile = "minimal"
```

### 2. **.cargo/config.toml**
Enforces consistent build settings and lint rules:
- Sets `rustflags = ["-D", "warnings"]` to deny warnings
- Configures clippy lints (e.g., `uninlined_format_args = "deny"`)
- Defines build profiles

### 3. **.clippy.toml**
Sets clippy configuration parameters:
- MSRV = "1.86.0"
- Complexity thresholds
- Function size limits

### 4. **rustfmt.toml**
Ensures consistent code formatting:
- Line width = 100
- Unix line endings
- Consistent function parameter layout

### 5. **Git Hooks (.githooks/)**
Pre-commit checks that run automatically:
- Format checking
- Clippy lints
- Test execution

### 6. **Make Commands**
- `make ci` - Quick format and lint checks
- `make ci-local` - Exactly mirrors what CI runs
- `make audit` - Security vulnerability scanning

## Setup Instructions

For new developers:
```bash
./scripts/setup-dev.sh
```

This script:
1. Configures git hooks
2. Installs required tools (cargo-watch, cargo-audit)
3. Verifies the correct Rust toolchain

## Troubleshooting

If you see differences between local and CI:

1. **Check Rust version**: `rustup show`
2. **Clean build**: `cargo clean`
3. **Update dependencies**: `cargo update`
4. **Run full CI locally**: `make ci-local`

## CI Workflows

### Main CI Workflow (.github/workflows/ci.yml)
- Uses `actions-rust-lang/setup-rust-toolchain@v1` which respects `rust-toolchain.toml`
- Runs the exact same commands as `make ci-local`
- No more testing against beta/nightly - only our pinned stable version
- Includes cross-platform testing (Windows, macOS, Linux)
- Verifies MSRV matches rust-toolchain.toml

### PR Checks Workflow (.github/workflows/pr-checks.yml)
- Lightweight checks for pull requests
- Quick formatting and linting checks run first
- Tests only run if quick checks pass
- Helps catch issues early in the PR process

## Why This Matters

- **No Surprises**: Code that passes locally will pass in CI
- **Consistent Style**: Everyone's code looks the same
- **Early Detection**: Issues caught before pushing
- **Team Efficiency**: Less time debugging CI failures
- **Version Consistency**: CI uses the exact same Rust version as local development