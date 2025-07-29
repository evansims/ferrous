# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Comprehensive test suite with unit and integration tests
- Input validation framework with sanitization
- Rate limiting middleware with configurable limits
- Request ID tracking for debugging
- Graceful shutdown with connection draining
- Security headers middleware
- Enhanced health endpoints (liveness/readiness)
- Prometheus metrics endpoint
- API versioning strategy
- OpenAPI documentation
- Structured error responses
- Configuration validation at startup
- Secrets management with rotation support
- JWKS authentication support
- Deployment guide with container support
- Contributing guide and issue templates

### Changed
- Improved main function error handling
- Enhanced database abstraction with metrics
- Updated all dependencies to latest versions

### Security
- Added security headers (CSP, HSTS, etc.)
- Implemented secrets scanning in CI
- Added authentication middleware with JWT support

## [0.1.0] - 2024-01-01

### Added
- Initial release
- Basic REST API for items management
- In-memory database implementation
- Convex database support
- CORS support
- Basic health check endpoint
- Environment-based configuration

[Unreleased]: https://github.com/evansims/ferrous/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/evansims/ferrous/releases/tag/v0.1.0
