# Development Plan

This document tracks planned improvements and enhancements for the Estuary project.

## 🚨 Priority 1: Critical Issues

### 1. Add Comprehensive Test Suite
**Status**: Partially Complete
**Why**: Currently no tests exist despite CI running `cargo test`
- [x] Unit tests for database implementations (in-memory, ~~Convex~~)
- [x] Integration tests for all API endpoints
- [x] Repository pattern tests
- [x] Error handling and edge case tests
- [x] Test utilities and fixtures
**Note**: Convex database tests pending due to external dependency requirements

### 2. Improve Main Function Error Handling
**Status**: Completed ✅
**Why**: Current `.expect()` calls will panic in production
- [x] Refactor main.rs to return `Result<(), Box<dyn Error>>`
- [x] Add graceful error messages for configuration issues
- [x] Handle database initialization failures gracefully
- [x] Improve server startup error reporting

## 🔧 Priority 2: Important Features

### 3. Input Validation Framework
**Status**: Completed ✅
**Why**: No validation beyond basic type checking
- [x] Add field length limits (max 255 for name, 1000 for description)
- [x] Implement format validation for fields
- [x] Add input sanitization
- [x] Create reusable validation middleware
- [x] Update tests to cover validation

### 4. Rate Limiting
**Status**: Completed ✅
**Why**: Protect against abuse and ensure fair usage
- [x] Add rate limiting middleware using tower-http
- [x] Configure per-IP rate limits
- [x] Add rate limit headers to responses
- [x] Document rate limits in API docs
- [x] **Note**: Default to permissive limits in development (e.g., 1000 req/min) with easy env var overrides
- [x] Update tests to cover rate limiting

### 5. Request ID Tracking
**Status**: Completed ✅
**Why**: Essential for debugging and log correlation
- [x] Add request ID middleware
- [x] Include request ID in all log entries
- [x] Return request ID in response headers
- [x] Update error responses to include request ID
- [x] Update tests to cover request ID tracking

### 6. Graceful Shutdown
**Status**: Completed ✅
**Why**: Prevent data loss and connection drops during deployment
- [x] Implement signal handling (SIGTERM, SIGINT)
- [x] Add shutdown timeout configuration
- [x] Drain existing connections before shutdown
- [x] Log shutdown process

### 7. Security Headers
**Status**: Completed ✅
**Why**: Basic security best practice
- [x] Add security headers middleware
- [x] Configure CSP, X-Frame-Options, X-Content-Type-Options
- [x] Add HSTS header for HTTPS deployments
- [x] Document security headers
- [x] **Note**: Default to permissive CSP in development mode with stricter production defaults
- [x] Update tests to cover security headers

## 📊 Priority 3: Observability

### 8. Enhanced Health Endpoint
**Status**: Completed ✅
**Why**: Current health check is too basic for production
- [x] Add database connectivity check
- [x] Include memory usage stats
- [x] Add uptime information
- [x] Include version and build info
- [x] Create separate liveness and readiness endpoints

### 9. Metrics and Monitoring
**Status**: Completed ✅
**Why**: No visibility into application performance
- [x] Add Prometheus metrics endpoint at /metrics
- [x] Track request duration, status codes
- [x] Monitor database query performance
- [x] Add custom business metrics (items created/updated/deleted)

## 📝 Priority 4: Developer Experience

### 10. API Versioning Strategy
**Status**: Not Started
**Why**: Need clear strategy for future API evolution
- [ ] Document versioning approach
- [ ] Add version negotiation
- [ ] Plan deprecation strategy
- [ ] Update API documentation

### 11. OpenAPI Documentation
**Status**: Completed ✅
**Why**: Machine-readable API documentation improves adoption
- [x] Add OpenAPI spec generation
- [x] Create OpenAPI JSON endpoint at /openapi.json
- [x] Keep spec in sync with code
- [x] Add request/response examples

### 12. Structured Error Responses
**Status**: Completed ✅
**Why**: Consistent error format improves client integration
- [x] Define standard error response schema
- [x] Add error codes beyond HTTP status
- [x] Include helpful error details
- [x] Document all error scenarios

## 🔒 Priority 5: Configuration & Security

### 13. Configuration Validation
**Status**: Not Started
**Why**: Fail fast on misconfiguration
- [ ] Validate all env vars at startup
- [ ] Provide helpful error messages
- [ ] Add configuration schema
- [ ] Support configuration profiles

### 14. Secrets Management
**Status**: Not Started
**Why**: Prepare for production deployments
- [ ] Document secrets management approach
- [ ] Add support for secret rotation
- [ ] Implement secure defaults
- [ ] Add secrets scanning to CI

### 15. JWKS Authentication
**Status**: Completed ✅
**Why**: Modern, secure API authentication
- [x] Add JWKS (JSON Web Key Set) endpoint support for validating JWTs
- [x] Support multiple JWKS URLs for different clients/tenants
- [x] Implement private key JWT client assertions (RFC 7523)
- [x] Add token validation middleware with configurable claims
- [x] Support key rotation via JWKS endpoint polling
- [x] Make authentication optional in development mode
- [x] Document authentication setup and client examples

## 📚 Priority 6: Documentation

### 16. Deployment Guide
**Status**: Not Started
**Why**: Help users deploy to production
- [ ] Create deployment best practices
- [ ] Add container deployment guide
- [ ] Document scaling considerations
- [ ] Include monitoring setup

### 17. Contributing Guide
**Status**: Not Started
**Why**: Encourage community contributions
- [ ] Create CONTRIBUTING.md
- [ ] Define code style guidelines
- [ ] Document PR process
- [ ] Add issue templates

## Implementation Order

1. **Phase 1** (Foundation): Tests (#1), Error Handling (#2) ✅
2. **Phase 2** (Security): Input Validation (#3), Rate Limiting (#4), Security Headers (#7), JWKS Auth (#15) ✅
3. **Phase 3** (Operations): Graceful Shutdown (#6), Health Endpoints (#8), Request Tracking (#5), Metrics (#9) ✅
4. **Phase 4** (Developer Experience): OpenAPI (#11), Error Responses (#12) ✅
5. **Phase 5** (Production Ready): Config Validation (#13), Deployment Docs (#16)

## Development Philosophy

### Developer-Friendly Defaults
- **Rate Limiting**: Default to very permissive limits (1000+ requests/minute) in development
- **Security Headers**: Use relaxed CSP policies in development mode
- **Authentication**: Make JWT/JWKS validation optional in development (controlled by env var)
- **All security features**: Should be easily toggled via environment variables

### Production-Ready Security
- Support enterprise authentication patterns (JWKS, private key assertions)
- Allow for strict security policies when explicitly configured
- Provide clear documentation for transitioning from dev to production settings

## Notes

- Each item should be implemented as a separate PR
- Update this file as items are completed
- Consider creating GitHub issues for tracking
- Prioritize based on immediate needs
- Always maintain backward compatibility when adding new features
