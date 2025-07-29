# API Reference

This document describes the REST API endpoints provided by Estuary.

## Base URL

```
http://localhost:3000
```

## Documentation

### GET /openapi.json

OpenAPI 3.0 specification for the API in JSON format.

**Response**
- JSON document conforming to OpenAPI 3.0 specification
- Content-Type: `application/json`

**Example Usage**
```bash
curl http://localhost:3000/openapi.json
```

## Health Checks

### GET /

Basic health check endpoint.

**Response**
```json
{
  "status": "healthy",
  "timestamp": "2024-01-15T10:30:00Z"
}
```

**Status Codes**
- `200 OK` - Service is healthy

### GET /health

Comprehensive health check with system and database metrics.

**Response**
```json
{
  "status": "healthy",
  "timestamp": "2024-01-15T10:30:00Z",
  "uptime_seconds": 3600,
  "version": "0.1.0",
  "database": {
    "connected": true,
    "response_time_ms": 5
  },
  "system": {
    "memory_used_mb": 1024,
    "memory_total_mb": 8192,
    "memory_usage_percent": 12.5,
    "cpu_count": 8
  }
}
```

**Status Values**
- `healthy` - All systems operational
- `degraded` - Service operational but with high resource usage (>90% memory)
- `unhealthy` - Database connection failed

**Status Codes**
- `200 OK` - Service is operational
- `500 Internal Server Error` - Error retrieving health status

### GET /health/live

Liveness probe for container orchestration systems.

**Response**
```json
{
  "status": "alive",
  "timestamp": "2024-01-15T10:30:00Z"
}
```

**Status Codes**
- `200 OK` - Service is alive

### GET /health/ready

Readiness probe that checks database connectivity.

**Response (Ready)**
```json
{
  "status": "ready",
  "timestamp": "2024-01-15T10:30:00Z"
}
```

**Response (Not Ready)**
```json
{
  "status": "not_ready",
  "timestamp": "2024-01-15T10:30:00Z",
  "reason": "database_unavailable"
}
```

**Status Codes**
- `200 OK` - Service is ready to accept requests
- `503 Service Unavailable` - Service is not ready (database unavailable)

## Items API

### List Items

**GET** `/api/v1/items`

Retrieve a paginated list of items.

**Query Parameters**
- `limit` (optional, default: 20, max: 100) - Number of items to return
- `offset` (optional, default: 0) - Number of items to skip

**Response**
```json
{
  "items": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "name": "Example Item",
      "description": "This is an example item",
      "created_at": "2024-01-15T10:00:00Z",
      "updated_at": "2024-01-15T10:00:00Z"
    }
  ],
  "total": 42,
  "limit": 10,
  "offset": 0
}
```

**Status Codes**
- `200 OK` - Success
- `400 Bad Request` - Invalid query parameters
- `500 Internal Server Error` - Server error

### Get Item

**GET** `/api/v1/items/{id}`

Retrieve a single item by ID.

**Path Parameters**
- `id` - The item's unique identifier

**Response**
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "name": "Example Item",
  "description": "This is an example item",
  "created_at": "2024-01-15T10:00:00Z",
  "updated_at": "2024-01-15T10:00:00Z"
}
```

**Status Codes**
- `200 OK` - Success
- `404 Not Found` - Item not found
- `500 Internal Server Error` - Server error

### Create Item

**POST** `/api/v1/items`

Create a new item.

**Request Body**
```json
{
  "name": "New Item",
  "description": "Optional description"
}
```

**Request Fields**
- `name` (required, string, 1-255 characters) - The item name
- `description` (optional, string, max 1000 characters) - The item description

**Validation Rules**
- Name must be between 1 and 255 characters
- Description must not exceed 1000 characters
- Input is automatically trimmed of whitespace
- Empty strings are converted to null for optional fields

**Response**
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "name": "New Item",
  "description": "Optional description",
  "created_at": "2024-01-15T10:00:00Z",
  "updated_at": "2024-01-15T10:00:00Z"
}
```

**Status Codes**
- `201 Created` - Item created successfully
- `400 Bad Request` - Invalid request body
- `422 Unprocessable Entity` - Validation error
- `500 Internal Server Error` - Server error

### Update Item

**PUT** `/api/v1/items/{id}`

Update an existing item.

**Path Parameters**
- `id` - The item's unique identifier

**Request Body**
```json
{
  "name": "Updated Name",
  "description": "Updated description"
}
```

**Request Fields**
- `name` (optional, string, 1-255 characters) - The updated item name
- `description` (optional, string, max 1000 characters) - The updated item description

**Validation Rules**
- Name must be between 1 and 255 characters (if provided)
- Description must not exceed 1000 characters (if provided)
- Input is automatically trimmed of whitespace
- Empty strings are converted to null for optional fields

**Response**
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "name": "Updated Name",
  "description": "Updated description",
  "created_at": "2024-01-15T10:00:00Z",
  "updated_at": "2024-01-15T11:00:00Z"
}
```

**Status Codes**
- `200 OK` - Item updated successfully
- `400 Bad Request` - Invalid request body
- `404 Not Found` - Item not found
- `422 Unprocessable Entity` - Validation error
- `500 Internal Server Error` - Server error

### Delete Item

**DELETE** `/api/v1/items/{id}`

Delete an item.

**Path Parameters**
- `id` - The item's unique identifier

**Response**
```
204 No Content
```

**Status Codes**
- `204 No Content` - Item deleted successfully
- `404 Not Found` - Item not found
- `500 Internal Server Error` - Server error

## Error Responses

All error responses follow a consistent structured format:

```json
{
  "error": "VALIDATION_ERROR",
  "message": "Validation failed",
  "details": {
    "validation_errors": [
      {
        "field": "name",
        "message": "Name must be between 1 and 255 characters",
        "code": "length"
      }
    ],
    "context": "Additional error context"
  },
  "timestamp": "2024-01-15T10:30:00Z",
  "request_id": "550e8400-e29b-41d4-a716-446655440000"
}
```

### Error Codes

- `BAD_REQUEST` - Invalid request format or parameters
- `VALIDATION_ERROR` - Request validation failed
- `NOT_FOUND` - Resource not found
- `UNAUTHORIZED` - Authentication required or invalid token
- `FORBIDDEN` - Authenticated but not authorized for this resource
- `RATE_LIMIT_EXCEEDED` - Too many requests
- `INTERNAL_SERVER_ERROR` - Internal server error
- `DATABASE_ERROR` - Database operation failed
- `LOCK_ERROR` - Failed to acquire resource lock
- `SERVICE_UNAVAILABLE` - Service temporarily unavailable

## Rate Limiting

Rate limiting is enabled by default to protect against abuse.

### Default Limits
- **Development**: 1000 requests per minute per IP address
- **Production**: Configure via `RATE_LIMIT_MAX_REQUESTS` and `RATE_LIMIT_WINDOW_SECONDS`

### Rate Limit Headers

All responses include rate limit information:
- `X-RateLimit-Limit` - Maximum requests allowed in the window
- `X-RateLimit-Remaining` - Requests remaining in current window
- `X-RateLimit-Reset` - Unix timestamp when the window resets

### Rate Limit Response

When rate limit is exceeded:
```json
{
  "error": "RATE_LIMIT_EXCEEDED",
  "message": "Too many requests",
  "timestamp": "2024-01-15T10:30:00Z",
  "request_id": "550e8400-e29b-41d4-a716-446655440000"
}
```

**Status Code**: `429 Too Many Requests`

## Authentication

JWT authentication using JWKS (JSON Web Key Sets) is supported.

### Configuration
- **Development**: Authentication is disabled by default (`AUTH_ENABLED=false`)
- **Production**: Enable with `AUTH_ENABLED=true` and configure JWKS endpoints

### JWT Requirements

When authentication is enabled, requests must include:
```
Authorization: Bearer <JWT token>
```

### Token Validation
- Tokens are validated against configured JWKS endpoints
- Supports multiple JWKS URLs for multi-tenant scenarios
- Automatic key rotation via JWKS endpoint polling
- Configurable audience and issuer validation

### Authentication Errors

**Missing Token**
```json
{
  "error": "UNAUTHORIZED",
  "message": "Authentication required",
  "timestamp": "2024-01-15T10:30:00Z"
}
```

**Invalid Token**
```json
{
  "error": "UNAUTHORIZED",
  "message": "Invalid authentication token",
  "timestamp": "2024-01-15T10:30:00Z"
}
```

## CORS

CORS is enabled with permissive settings for development. Production deployments should configure appropriate CORS origins.

## Headers

### Request Headers

- `Content-Type: application/json` - Required for requests with bodies
- `Authorization: Bearer <token>` - Required when authentication is enabled

### Response Headers

#### Standard Headers
- `Content-Type: application/json` - All responses are JSON
- `X-Request-Id` - Unique request identifier for correlation

#### Security Headers
- `X-Content-Type-Options: nosniff` - Prevent MIME sniffing
- `X-Frame-Options: DENY` - Prevent clickjacking
- `X-XSS-Protection: 1; mode=block` - XSS protection (legacy browsers)
- `Referrer-Policy: no-referrer` - Privacy protection
- `Content-Security-Policy` - Configurable CSP headers
- `Strict-Transport-Security` - HSTS for HTTPS deployments

#### Rate Limiting Headers
- `X-RateLimit-Limit` - Max requests per window
- `X-RateLimit-Remaining` - Remaining requests
- `X-RateLimit-Reset` - Window reset timestamp

## Versioning

The API is versioned via the URL path. The current version is `v1`.

Example: `/api/v1/items`

Future versions will be available at `/api/v2/items`, etc.

## Metrics

### GET /metrics

Prometheus-compatible metrics endpoint that exposes application performance and business metrics.

**Response**
- Content-Type: `text/plain; version=0.0.4`
- Prometheus text format metrics

**Available Metrics**

#### HTTP Metrics
- `http_request_duration_seconds` - HTTP request duration histogram by method, endpoint, and status
- `http_requests_total` - Total number of HTTP requests by method, endpoint, and status

#### Database Metrics
- `database_query_duration_seconds` - Database query duration histogram by operation and repository
- `database_queries_total` - Total number of database queries by operation, repository, and status
- `database_connections_active` - Number of active database connections (gauge)

#### Business Metrics
- `items_created_total` - Total number of items created
- `items_updated_total` - Total number of items updated
- `items_deleted_total` - Total number of items deleted

**Example Usage**
```bash
# Get current metrics
curl http://localhost:3000/metrics

# Example output
# HELP http_requests_total Total number of HTTP requests
# TYPE http_requests_total counter
http_requests_total{endpoint="/api/v1/items",method="GET",status="200"} 42
```

**Integration with Prometheus**

Add to your `prometheus.yml`:
```yaml
scrape_configs:
  - job_name: 'estuary'
    static_configs:
      - targets: ['localhost:3000']
```

**Note**: The metrics endpoint bypasses authentication when enabled to allow monitoring systems to scrape metrics without credentials.

## Environment Configuration

The API behavior can be configured via environment variables. See `.env.example` for all available options.

### Key Configuration Options

#### Authentication
- `AUTH_ENABLED` - Enable/disable JWT authentication (default: `false`)
- `AUTH_JWKS_URLS` - Comma-separated list of JWKS endpoints
- `AUTH_AUDIENCE` - Expected JWT audience claim
- `AUTH_ISSUER` - Expected JWT issuer claim

#### Rate Limiting
- `RATE_LIMIT_ENABLED` - Enable/disable rate limiting (default: `true`)
- `RATE_LIMIT_MAX_REQUESTS` - Max requests per window (default: `1000`)
- `RATE_LIMIT_WINDOW_SECONDS` - Time window in seconds (default: `60`)

#### Security
- `SECURITY_STRICT_MODE` - Enable strict security headers (default: `false`)
- `SECURITY_CSP` - Custom Content Security Policy header

#### Server
- `PORT` - Server port (default: `3000`)
- `SHUTDOWN_TIMEOUT_SECONDS` - Graceful shutdown timeout (default: `30`)