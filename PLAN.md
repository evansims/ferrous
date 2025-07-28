# Development Plan

This document tracks planned improvements and enhancements for the Estuary project.

## 🚨 Priority 1: Critical Issues

### 1. Add Comprehensive Test Suite
**Status**: Not Started  
**Why**: Currently no tests exist despite CI running `cargo test`
- [ ] Unit tests for database implementations (in-memory, Convex)
- [ ] Integration tests for all API endpoints
- [ ] Repository pattern tests
- [ ] Error handling and edge case tests
- [ ] Test utilities and fixtures

### 2. Improve Main Function Error Handling
**Status**: Not Started  
**Why**: Current `.expect()` calls will panic in production
- [ ] Refactor main.rs to return `Result<(), Box<dyn Error>>`
- [ ] Add graceful error messages for configuration issues
- [ ] Handle database initialization failures gracefully
- [ ] Improve server startup error reporting

## 🔧 Priority 2: Important Features

### 3. Input Validation Framework
**Status**: Not Started  
**Why**: No validation beyond basic type checking
- [ ] Add field length limits (max 255 for name, 1000 for description)
- [ ] Implement format validation for fields
- [ ] Add input sanitization
- [ ] Create reusable validation middleware

### 4. Rate Limiting
**Status**: Not Started  
**Why**: Protect against abuse and ensure fair usage
- [ ] Add rate limiting middleware using tower-http
- [ ] Configure per-IP rate limits
- [ ] Add rate limit headers to responses
- [ ] Document rate limits in API docs
- [ ] **Note**: Default to permissive limits in development (e.g., 1000 req/min) with easy env var overrides

### 5. Request ID Tracking
**Status**: Not Started  
**Why**: Essential for debugging and log correlation
- [ ] Add request ID middleware
- [ ] Include request ID in all log entries
- [ ] Return request ID in response headers
- [ ] Update error responses to include request ID

### 6. Graceful Shutdown
**Status**: Not Started  
**Why**: Prevent data loss and connection drops during deployment
- [ ] Implement signal handling (SIGTERM, SIGINT)
- [ ] Add shutdown timeout configuration
- [ ] Drain existing connections before shutdown
- [ ] Log shutdown process

### 7. Security Headers
**Status**: Not Started  
**Why**: Basic security best practice
- [ ] Add security headers middleware
- [ ] Configure CSP, X-Frame-Options, X-Content-Type-Options
- [ ] Add HSTS header for HTTPS deployments
- [ ] Document security headers
- [ ] **Note**: Default to permissive CSP in development mode with stricter production defaults

## 📊 Priority 3: Observability

### 8. Enhanced Health Endpoint
**Status**: Not Started  
**Why**: Current health check is too basic for production
- [ ] Add database connectivity check
- [ ] Include memory usage stats
- [ ] Add uptime information
- [ ] Include version and build info
- [ ] Create separate liveness and readiness endpoints

### 9. Metrics and Monitoring
**Status**: Not Started  
**Why**: No visibility into application performance
- [ ] Add Prometheus metrics endpoint
- [ ] Track request duration, status codes
- [ ] Monitor database query performance
- [ ] Add custom business metrics

## 📝 Priority 4: Developer Experience

### 10. API Versioning Strategy
**Status**: Not Started  
**Why**: Need clear strategy for future API evolution
- [ ] Document versioning approach
- [ ] Add version negotiation
- [ ] Plan deprecation strategy
- [ ] Update API documentation

### 11. OpenAPI/Swagger Documentation
**Status**: Not Started  
**Why**: Interactive API documentation improves adoption
- [ ] Add OpenAPI spec generation
- [ ] Create Swagger UI endpoint
- [ ] Keep spec in sync with code
- [ ] Add request/response examples

### 12. Structured Error Responses
**Status**: Not Started  
**Why**: Consistent error format improves client integration
- [ ] Define standard error response schema
- [ ] Add error codes beyond HTTP status
- [ ] Include helpful error details
- [ ] Document all error scenarios

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
**Status**: Not Started  
**Why**: Modern, secure API authentication
- [ ] Add JWKS (JSON Web Key Set) endpoint support for validating JWTs
- [ ] Support multiple JWKS URLs for different clients/tenants
- [ ] Implement private key JWT client assertions (RFC 7523)
- [ ] Add token validation middleware with configurable claims
- [ ] Support key rotation via JWKS endpoint polling
- [ ] Make authentication optional in development mode
- [ ] Document authentication setup and client examples

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

1. **Phase 1** (Foundation): Tests (#1), Error Handling (#2)
2. **Phase 2** (Security): Input Validation (#3), Rate Limiting (#4), Security Headers (#7), JWKS Auth (#15)
3. **Phase 3** (Operations): Graceful Shutdown (#6), Health Endpoints (#8), Request Tracking (#5)
4. **Phase 4** (Developer Experience): OpenAPI (#11), Error Responses (#12)
5. **Phase 5** (Production Ready): Config Validation (#13), Metrics (#9), Deployment Docs (#16)

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