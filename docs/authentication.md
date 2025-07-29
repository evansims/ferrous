# Authentication Guide

Estuary supports JWT (JSON Web Token) authentication for secure API access.

## Overview

When authentication is enabled, all API endpoints (except health checks and documentation) require a valid JWT token. Tokens are validated using a shared secret key.

## Configuration

### Environment Variables

```bash
# Enable/disable authentication (default: false in development)
AUTH_ENABLED=true

# JWT secret key for token validation
JWT_SECRET=your-secret-key-here
```

### Development Mode

Authentication is disabled by default in development to simplify local testing. To test authentication locally:

```bash
AUTH_ENABLED=true JWT_SECRET=test-secret cargo run
```

## Making Authenticated Requests

Include the JWT token in the Authorization header:

```bash
curl -H "Authorization: Bearer YOUR_JWT_TOKEN" \
     http://localhost:3000/api/v1/items
```

## Token Requirements

### Required Claims

- `sub` (subject) - The user identifier
- `exp` (expiration) - Token expiration timestamp

### Token Structure

```json
{
  "sub": "user123",
  "exp": 1735430400
}
```

## Generating Test Tokens

For development and testing, you can generate JWT tokens using various tools:

### Using jwt.io

1. Go to [jwt.io](https://jwt.io)
2. Select HS256 algorithm
3. Enter your payload with required claims
4. Enter your JWT_SECRET in the signature field
5. Copy the generated token

### Using CLI (requires jwt-cli)

```bash
# Install jwt-cli
cargo install jwt-cli

# Generate a token
jwt encode --secret "your-secret-key" '{"sub":"user123","exp":1735430400}'
```

## Error Responses

### 401 Unauthorized

Returned when:
- No token is provided
- Token is invalid or expired
- Token signature verification fails

```json
{
  "error": "UNAUTHORIZED",
  "message": "Authentication required",
  "code": "E004",
  "request_id": "550e8400-e29b-41d4-a716-446655440000"
}
```

## Security Best Practices

1. **Strong Secrets**: Use a cryptographically secure random string for JWT_SECRET
2. **Token Expiration**: Always include and validate the `exp` claim
3. **HTTPS Only**: Always use HTTPS in production to prevent token interception
4. **Secret Rotation**: Regularly rotate your JWT secret in production
5. **Environment Security**: Never commit JWT_SECRET to version control

## Future Enhancements

The current implementation uses symmetric key validation (HS256). Future versions may support:
- JWKS (JSON Web Key Set) for key rotation
- Multiple identity providers
- RS256 (asymmetric) validation
- OAuth 2.0 integration