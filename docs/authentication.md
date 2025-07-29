# Authentication Guide

Estuary supports JWT (JSON Web Token) authentication with JWKS (JSON Web Key Set) validation, enabling secure API access with support for key rotation and multiple identity providers.

## Overview

When authentication is enabled, all API endpoints (except health checks and documentation) require a valid JWT token. Tokens are validated against one or more JWKS endpoints, supporting enterprise authentication patterns including private key JWT client assertions (RFC 7523).

## Configuration

### Environment Variables

```bash
# Enable/disable authentication (default: false in development)
AUTH_ENABLED=true

# JWKS endpoints (comma-separated for multiple providers)
AUTH_JWKS_URLS=https://your-auth0-domain.auth0.com/.well-known/jwks.json

# Expected audience claim (optional)
AUTH_AUDIENCE=https://api.yourdomain.com

# Expected issuer claim (optional)
AUTH_ISSUER=https://your-auth0-domain.auth0.com/

# JWKS cache duration in seconds (default: 3600)
AUTH_JWKS_CACHE_SECONDS=3600
```

### Development Mode

Authentication is disabled by default in development to simplify local testing. To test authentication locally:

```bash
AUTH_ENABLED=true AUTH_JWKS_URLS=https://example.com/jwks.json cargo run
```

## Making Authenticated Requests

Include the JWT token in the Authorization header:

```bash
curl -H "Authorization: Bearer YOUR_JWT_TOKEN" \
     http://localhost:3000/api/v1/items
```

## Token Requirements

### Required Claims

- `exp` - Expiration time (tokens must not be expired)
- `iat` - Issued at time (optional but recommended)

### Optional Claims (if configured)

- `aud` - Audience (must match `AUTH_AUDIENCE` if set)
- `iss` - Issuer (must match `AUTH_ISSUER` if set)
- `sub` - Subject (logged but not validated)

## Multiple JWKS Providers

Estuary supports validating tokens from multiple identity providers simultaneously:

```bash
AUTH_JWKS_URLS=https://provider1.com/jwks.json,https://provider2.com/jwks.json
```

This is useful for:
- Multi-tenant applications
- Migrating between identity providers
- Supporting both user and service account authentication

## Private Key JWT Authentication

Estuary supports OAuth 2.0 private key JWT client authentication (RFC 7523), where clients authenticate using JWTs signed with their private keys:

1. Client generates a JWT signed with their private key
2. Client's public key is registered in the JWKS endpoint
3. Estuary validates the JWT signature using the JWKS endpoint

## Key Rotation

JWKS endpoints are cached for performance but regularly refreshed to support key rotation:

- Default cache duration: 1 hour (3600 seconds)
- Configure with `AUTH_JWKS_CACHE_SECONDS`
- On validation failure, cache is refreshed immediately to handle emergency key rotation

## Error Responses

### Missing Token

```json
{
  "error": "UNAUTHORIZED",
  "message": "Authentication required",
  "timestamp": "2024-01-15T10:30:00Z",
  "request_id": "550e8400-e29b-41d4-a716-446655440000"
}
```

### Invalid Token

```json
{
  "error": "UNAUTHORIZED",
  "message": "Invalid authentication token",
  "details": {
    "context": "Token signature validation failed"
  },
  "timestamp": "2024-01-15T10:30:00Z",
  "request_id": "550e8400-e29b-41d4-a716-446655440000"
}
```

### Expired Token

```json
{
  "error": "UNAUTHORIZED",
  "message": "Token has expired",
  "timestamp": "2024-01-15T10:30:00Z",
  "request_id": "550e8400-e29b-41d4-a716-446655440000"
}
```

## Testing with Auth0

Example configuration for Auth0:

```bash
AUTH_ENABLED=true
AUTH_JWKS_URLS=https://YOUR_DOMAIN.auth0.com/.well-known/jwks.json
AUTH_AUDIENCE=https://api.yourdomain.com
AUTH_ISSUER=https://YOUR_DOMAIN.auth0.com/
```

## Testing with Local JWKS

For testing, you can use a local JWKS endpoint:

1. Generate a key pair
2. Create a JWKS endpoint serving your public key
3. Sign JWTs with your private key
4. Configure Estuary to use your local JWKS endpoint

## Security Best Practices

1. **Always use HTTPS** in production for JWKS endpoints
2. **Validate audience claims** to prevent token reuse across services
3. **Keep cache duration reasonable** - balance between performance and security
4. **Monitor authentication failures** in logs for security incidents
5. **Use short-lived tokens** and implement token refresh on the client side

## Troubleshooting

### Enable Debug Logging

```bash
RUST_LOG=estuary=debug cargo run
```

### Common Issues

1. **"Invalid authentication token"** - Check that the token is properly formatted and signed
2. **"Token has expired"** - Ensure your system clock is synchronized
3. **"JWKS endpoint unreachable"** - Verify network connectivity and JWKS URL
4. **"Invalid audience"** - Token audience doesn't match `AUTH_AUDIENCE`

## Example: Client Implementation

### Node.js with jsonwebtoken

```javascript
const jwt = require('jsonwebtoken');
const axios = require('axios');

// Generate token (usually done by your auth provider)
const token = jwt.sign(
  { 
    sub: 'user123',
    aud: 'https://api.yourdomain.com'
  },
  privateKey,
  { 
    algorithm: 'RS256',
    expiresIn: '1h',
    issuer: 'https://your-auth-domain.com/'
  }
);

// Make authenticated request
const response = await axios.get('http://localhost:3000/api/v1/items', {
  headers: {
    'Authorization': `Bearer ${token}`
  }
});
```

### Python with requests

```python
import requests

# Token obtained from your auth provider
token = "YOUR_JWT_TOKEN"

response = requests.get(
    "http://localhost:3000/api/v1/items",
    headers={"Authorization": f"Bearer {token}"}
)
```