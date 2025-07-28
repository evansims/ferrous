# API Reference

This document describes the REST API endpoints provided by Estuary.

## Base URL

```
http://localhost:3000
```

## Health Check

### GET /

Check if the service is running and healthy.

**Response**
```json
{
  "status": "healthy",
  "timestamp": "2024-01-15T10:30:00Z"
}
```

**Status Codes**
- `200 OK` - Service is healthy
- `503 Service Unavailable` - Service is unhealthy

## Items API

### List Items

**GET** `/api/v1/items`

Retrieve a paginated list of items.

**Query Parameters**
- `limit` (optional, default: 10, max: 100) - Number of items to return
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
- `name` (required, string) - The item name
- `description` (optional, string) - The item description

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
- `name` (optional, string) - The updated item name
- `description` (optional, string) - The updated item description

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

All error responses follow a consistent format:

```json
{
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "Name is required",
    "details": {
      "field": "name"
    }
  }
}
```

### Error Codes

- `VALIDATION_ERROR` - Request validation failed
- `NOT_FOUND` - Resource not found
- `INTERNAL_ERROR` - Internal server error
- `DATABASE_ERROR` - Database operation failed

## Rate Limiting

Currently, no rate limiting is implemented. This may be added in future versions.

## Authentication

Currently, no authentication is required. This will be added in future versions.

## CORS

CORS is enabled with permissive settings for development. Production deployments should configure appropriate CORS origins.

## Content Types

- All requests with bodies must include `Content-Type: application/json`
- All responses return `Content-Type: application/json`

## Versioning

The API is versioned via the URL path. The current version is `v1`.

Example: `/api/v1/items`

Future versions will be available at `/api/v2/items`, etc.