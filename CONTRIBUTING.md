# Contributing to Ferrous

Thank you for your interest in contributing to Ferrous! We welcome contributions from the community and are grateful for any help you can provide.

## Table of Contents

1. [Code of Conduct](#code-of-conduct)
2. [Getting Started](#getting-started)
3. [Development Setup](#development-setup)
4. [Code Style Guidelines](#code-style-guidelines)
5. [Making Changes](#making-changes)
6. [Pull Request Process](#pull-request-process)
7. [Issue Guidelines](#issue-guidelines)
8. [Testing](#testing)
9. [Documentation](#documentation)
10. [Community](#community)

## Code of Conduct

By participating in this project, you agree to abide by our Code of Conduct:

- **Be respectful**: Treat everyone with respect. No harassment, discrimination, or offensive behavior.
- **Be collaborative**: Work together to resolve conflicts and find solutions.
- **Be constructive**: Provide helpful feedback and be open to receiving it.
- **Be inclusive**: Welcome newcomers and help them get started.

## Getting Started

1. **Fork the repository** on GitHub
2. **Clone your fork** locally:
   ```bash
   git clone https://github.com/your-username/ferrous.git
   cd ferrous
   ```
3. **Add the upstream remote**:
   ```bash
   git remote add upstream https://github.com/evansims/ferrous.git
   ```

## Development Setup

### Prerequisites

- Rust 1.70 or later (install via [rustup](https://rustup.rs/))
- Git
- Make (optional but recommended)

### Initial Setup

1. **Install Rust**:
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **Install development tools**:
   ```bash
   rustup component add clippy rustfmt
   cargo install cargo-watch  # For hot-reloading during development
   ```

3. **Set up pre-commit hooks** (optional but recommended):
   ```bash
   pip install pre-commit
   pre-commit install
   ```

4. **Create environment file**:
   ```bash
   cp .env.example .env
   ```

### Running the Project

```bash
# Using Make (recommended)
make run       # Run in development mode
make watch     # Run with hot-reload
make test      # Run all tests
make lint      # Run linter

# Using Cargo directly
cargo run                  # Run the application
cargo watch -x run        # Run with hot-reload
cargo test                # Run tests
cargo clippy              # Run linter
cargo fmt                 # Format code
```

## Code Style Guidelines

We use standard Rust conventions and tools to maintain code quality:

### Formatting

- **Use `rustfmt`** for all code formatting:
  ```bash
  cargo fmt
  ```
- Configuration is in `rustfmt.toml`
- Format your code before committing

### Linting

- **Use `clippy`** for linting:
  ```bash
  cargo clippy -- -D warnings
  ```
- Fix all clippy warnings before submitting a PR
- If you must ignore a warning, add a comment explaining why

### Code Conventions

1. **Naming**:
   - Use `snake_case` for functions, variables, and modules
   - Use `CamelCase` for types and traits
   - Use `SCREAMING_SNAKE_CASE` for constants

2. **Error Handling**:
   - Use `Result<T, E>` for fallible operations
   - Implement custom error types using the `thiserror` crate
   - Avoid `unwrap()` except in tests or when impossible to fail

3. **Documentation**:
   - Document all public APIs with `///` comments
   - Include examples in documentation when helpful
   - Use `//!` for module-level documentation

4. **Testing**:
   - Write unit tests in the same file as the code
   - Write integration tests in the `tests/` directory
   - Aim for high test coverage
   - Test error cases, not just happy paths

5. **Dependencies**:
   - Minimize external dependencies
   - Prefer well-maintained, popular crates
   - Document why each dependency is needed

### Example Code

```rust
/// Represents an item in the system.
///
/// # Examples
///
/// ```
/// use ferrous::models::Item;
///
/// let item = Item {
///     id: "123".to_string(),
///     name: "Example".to_string(),
///     description: Some("An example item".to_string()),
///     created_at: chrono::Utc::now(),
///     updated_at: chrono::Utc::now(),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Item {
    /// Unique identifier for the item
    pub id: String,
    /// Name of the item
    pub name: String,
    /// Optional description
    pub description: Option<String>,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
}

impl Item {
    /// Creates a new item with the given name.
    ///
    /// # Errors
    ///
    /// Returns an error if the name is empty.
    pub fn new(name: String, description: Option<String>) -> Result<Self, ValidationError> {
        if name.is_empty() {
            return Err(ValidationError::EmptyName);
        }

        Ok(Self {
            id: Uuid::new_v4().to_string(),
            name,
            description,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_item_with_valid_name() {
        let item = Item::new("Test Item".to_string(), None).unwrap();
        assert_eq!(item.name, "Test Item");
        assert!(item.description.is_none());
    }

    #[test]
    fn test_new_item_with_empty_name() {
        let result = Item::new("".to_string(), None);
        assert!(result.is_err());
    }
}
```

## Making Changes

### Workflow

1. **Create a feature branch**:
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. **Make your changes**:
   - Write code
   - Add tests
   - Update documentation
   - Run tests and linting

3. **Commit your changes**:
   ```bash
   git add .
   git commit -m "feat: add new feature

   - Detailed description of what changed
   - Why the change was needed
   - Any breaking changes or migrations required"
   ```

### Commit Message Guidelines

We follow the [Conventional Commits](https://www.conventionalcommits.org/) specification:

- **feat**: New feature
- **fix**: Bug fix
- **docs**: Documentation changes
- **style**: Code style changes (formatting, etc.)
- **refactor**: Code refactoring
- **perf**: Performance improvements
- **test**: Adding or updating tests
- **chore**: Maintenance tasks

Examples:
```
feat: add rate limiting middleware
fix: correct validation for item names
docs: update API documentation
test: add integration tests for auth
```

## Pull Request Process

1. **Update your fork**:
   ```bash
   git fetch upstream
   git checkout main
   git merge upstream/main
   ```

2. **Rebase your feature branch**:
   ```bash
   git checkout feature/your-feature-name
   git rebase main
   ```

3. **Run all checks**:
   ```bash
   make test      # Run all tests
   make lint      # Run linter
   make fmt       # Format code
   ```

4. **Push to your fork**:
   ```bash
   git push origin feature/your-feature-name
   ```

5. **Create a Pull Request**:
   - Go to the GitHub repository
   - Click "New Pull Request"
   - Select your branch
   - Fill in the PR template
   - Link any related issues

### PR Requirements

- **Tests**: All tests must pass
- **Linting**: No clippy warnings
- **Formatting**: Code must be formatted with rustfmt
- **Documentation**: Update relevant documentation
- **Changelog**: Update CHANGELOG.md if applicable
- **Review**: At least one maintainer approval

### PR Template

```markdown
## Description
Brief description of what this PR does.

## Related Issues
Fixes #123

## Type of Change
- [ ] Bug fix
- [ ] New feature
- [ ] Breaking change
- [ ] Documentation update

## Testing
- [ ] Unit tests pass
- [ ] Integration tests pass
- [ ] Manual testing completed

## Checklist
- [ ] Code follows style guidelines
- [ ] Self-review completed
- [ ] Documentation updated
- [ ] Tests added/updated
- [ ] Breaking changes documented
```

## Issue Guidelines

### Creating Issues

Use the appropriate issue template:

1. **Bug Report**: For reporting bugs
2. **Feature Request**: For suggesting new features
3. **Documentation**: For documentation improvements
4. **Question**: For questions about the project

### Issue Template Example

```markdown
### Bug Report

**Description**
Clear description of the bug.

**To Reproduce**
1. Step one
2. Step two
3. See error

**Expected Behavior**
What should happen instead.

**Environment**
- OS: [e.g., Ubuntu 20.04]
- Rust version: [e.g., 1.70.0]
- Ferrous version: [e.g., 0.1.0]

**Additional Context**
Any other relevant information.
```

## Testing

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run tests with output
cargo test -- --nocapture

# Run tests in single thread (for env-dependent tests)
cargo test -- --test-threads=1
```

### Writing Tests

1. **Unit Tests**: Test individual functions and methods
2. **Integration Tests**: Test API endpoints and workflows
3. **Property Tests**: Use `proptest` for property-based testing
4. **Benchmarks**: Use `criterion` for performance testing

Example test:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;
    use tower::util::ServiceExt;

    #[tokio::test]
    async fn test_create_item() {
        let app = create_test_app().await;

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/items")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"name": "Test Item"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);
    }
}
```

## Documentation

### Code Documentation

- Document all public APIs
- Include examples where helpful
- Keep documentation up-to-date with code changes

### Project Documentation

- **README.md**: Project overview and quick start
- **CLAUDE.md**: Development guidelines
- **API.md**: API documentation
- **docs/**: Additional documentation

### Generating Documentation

```bash
# Generate and open documentation
cargo doc --open

# Generate documentation with private items
cargo doc --document-private-items
```

## Community

### Getting Help

- **GitHub Issues**: For bugs and feature requests
- **Discussions**: For questions and ideas
- **Discord**: Join our community chat (if available)

### Ways to Contribute

- **Code**: Fix bugs, add features, improve performance
- **Documentation**: Improve docs, add examples, fix typos
- **Testing**: Add tests, improve coverage, test on different platforms
- **Review**: Review PRs, provide feedback
- **Design**: Improve UI/UX, create diagrams
- **Community**: Help others, answer questions, write tutorials

## Recognition

Contributors will be recognized in:
- The CONTRIBUTORS file
- Release notes
- Project documentation

## Questions?

If you have questions about contributing:
1. Check existing documentation
2. Search closed issues
3. Ask in GitHub Discussions
4. Open a new issue with the Question template

Thank you for contributing to Ferrous! 🚀
