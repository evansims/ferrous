# API Versioning Strategy

This document outlines Estuary's API versioning strategy, including version negotiation, deprecation policies, and migration guidelines.

## Overview

Estuary uses a pragmatic approach to API versioning that balances stability for existing clients with the ability to evolve the API over time. We follow semantic versioning principles adapted for REST APIs.

## Versioning Scheme

### Version Format

API versions follow the format `v{MAJOR}` where MAJOR is a positive integer:
- `v1` - First stable version
- `v2` - Second major version with breaking changes
- etc.

### What Constitutes a Version Change

**Major Version Changes (Breaking Changes)**
- Removing endpoints or fields
- Changing the type of a field
- Changing authentication requirements
- Modifying the structure of request/response bodies
- Changing HTTP methods or status codes

**Non-Breaking Changes (No Version Change Required)**
- Adding new endpoints
- Adding optional fields to requests
- Adding fields to responses
- Adding new query parameters (with defaults)
- Adding new error codes
- Performance improvements

## Version Negotiation

Estuary supports three methods for API version negotiation, in order of precedence:

### 1. URL Path Versioning (Recommended)

The version is included in the URL path:
```
GET /api/v1/items
GET /api/v2/items
```

**Advantages:**
- Explicit and visible
- Easy to test and debug
- Works with all HTTP clients
- Clear in logs and metrics

### 2. Accept Header Versioning

Clients can specify the desired version using the Accept header:
```
Accept: application/vnd.estuary.v2+json
```

**Format:** `application/vnd.estuary.v{VERSION}+json`

**Advantages:**
- RESTful approach
- Keeps URLs stable
- Allows content negotiation

### 3. Custom Header Versioning

As a fallback, clients can use a custom header:
```
X-API-Version: 2
```

**Advantages:**
- Simple to implement
- Works with clients that can't modify Accept headers

## Default Behavior

- When no version is specified, the current stable version is used
- The current stable version is `v1`
- Requests to `/api/items` (without version) redirect to `/api/v1/items`

## Version Lifecycle

### 1. **Preview** (Optional)
- New major versions may have a preview period
- Marked as `v{VERSION}-preview`
- Not covered by stability guarantees
- May change before general availability

### 2. **Current**
- The recommended version for new integrations
- Fully supported with bug fixes and security updates
- Currently: `v1`

### 3. **Deprecated**
- Still functional but not recommended for new integrations
- Receives security fixes only
- Deprecation notices added to responses
- Minimum 6 months before removal

### 4. **Sunset**
- No longer available
- Returns 410 Gone status
- Includes migration information in error response

## Deprecation Process

### Timeline

1. **Announcement** (T-0)
   - Blog post and changelog entry
   - Email to registered developers
   - Deprecation headers added to responses

2. **Deprecation Period** (T+0 to T+6 months minimum)
   - Old version remains fully functional
   - Deprecation headers on all responses:
     ```
     Sunset: Sat, 1 Jan 2025 00:00:00 GMT
     Deprecation: true
     Link: <https://docs.estuary.com/migration/v2>; rel="successor-version"
     ```

3. **Final Warning** (T+5 months)
   - Additional warning headers
   - Increased communication frequency

4. **Sunset** (T+6 months)
   - Old version returns 410 Gone
   - Clear migration instructions in response

### Deprecation Response Headers

```http
HTTP/1.1 200 OK
Deprecation: true
Sunset: Sat, 1 Jan 2025 00:00:00 GMT
Link: <https://docs.estuary.com/migration/v2>; rel="successor-version"
Warning: 299 - "This API version is deprecated and will be removed on 2025-01-01"
```

## Migration Support

### Migration Guides

For each major version upgrade, we provide:
- Detailed changelog of breaking changes
- Code examples showing before/after
- Migration scripts where applicable
- Testing strategies

### Compatibility Mode

During deprecation periods, we may offer compatibility flags:
```
X-API-Compatibility: v1-response-format
```

This allows gradual migration of specific features.

## Implementation Details

### Version Constants

```rust
pub const API_VERSION_CURRENT: &str = "v1";
pub const API_VERSION_SUPPORTED: &[&str] = &["v1"];
pub const API_VERSION_DEPRECATED: &[&str] = &[];
```

### Version Extraction

The API extracts version information in this order:
1. URL path segment (e.g., `/api/v1/...`)
2. Accept header (e.g., `application/vnd.estuary.v1+json`)
3. X-API-Version header
4. Default to current version

### OpenAPI Documentation

Each API version has its own OpenAPI specification:
- `/openapi.json` - Current version
- `/api/v1/openapi.json` - Version 1 specification
- `/api/v2/openapi.json` - Version 2 specification (when available)

## Client Best Practices

### Always Specify Version

```bash
# Good - Explicit version
curl https://api.estuary.com/api/v1/items

# Avoid - Implicit version
curl https://api.estuary.com/api/items
```

### Handle Deprecation Headers

```python
response = requests.get('https://api.estuary.com/api/v1/items')
if response.headers.get('Deprecation') == 'true':
    sunset_date = response.headers.get('Sunset')
    logger.warning(f'API version will be removed on {sunset_date}')
```

### Test Against Multiple Versions

During migration periods, test against both old and new versions to ensure smooth transitions.

## Server Implementation

### Version Middleware

The server implements version extraction middleware that:
1. Parses the requested version
2. Validates against supported versions
3. Adds version to request context
4. Returns 404 for unsupported versions
5. Adds deprecation headers for deprecated versions

### Feature Flags

Internal feature flags control version-specific behavior:
```rust
if version >= Version::V2 {
    // New behavior
} else {
    // Legacy behavior
}
```

## FAQ

### Q: How long are old versions supported?
A: Minimum 6 months after deprecation announcement, often longer for major versions.

### Q: Can I use multiple API versions simultaneously?
A: Yes, during migration periods you can use different versions for different endpoints.

### Q: How are version-specific bugs handled?
A: 
- Current version: Fixed immediately
- Deprecated versions: Security fixes only
- Sunset versions: No fixes

### Q: What happens to unversioned endpoints?
A: They use the current stable version and may redirect to the versioned URL.

## Future Considerations

### Potential Enhancements

1. **GraphQL Support**: Version schema evolution
2. **WebSocket APIs**: Version negotiation for real-time endpoints
3. **Batch Operations**: Version mixing in batch requests
4. **A/B Testing**: Gradual rollout of new versions

### Version 2.0 Planning

When planning v2, we will:
1. Gather feedback from v1 usage patterns
2. Consolidate common feature requests
3. Modernize based on current best practices
4. Provide automated migration tools

## References

- [Semantic Versioning](https://semver.org/)
- [REST API Versioning Best Practices](https://www.ietf.org/rfc/rfc8631.html)
- [Sunset HTTP Header](https://tools.ietf.org/html/rfc8594)