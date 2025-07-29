# Secrets Management Guide

This document outlines best practices for managing secrets and sensitive configuration in Estuary.

## Overview

Estuary follows the principle of least privilege and secure-by-default configuration. All sensitive data should be provided through environment variables and never committed to the repository.

## What Are Secrets?

Secrets include any sensitive information that should not be exposed publicly:

- API keys and tokens
- Database credentials
- JWT signing keys
- JWKS URLs (if they contain authentication)
- Private certificates
- Encryption keys
- OAuth client secrets

## Environment Variables

### Required Secrets

The following environment variables may contain sensitive data:

```bash
# Database Configuration (when using external databases)
CONVEX_DEPLOYMENT_URL=https://your-deployment.convex.cloud

# Authentication Configuration
JWT_SECRET=your-secret-key-here

# Future: Additional secrets as needed
```

### Non-Sensitive Configuration

These environment variables are not considered secrets:

```bash
PORT=3000
DATABASE_TYPE=memory
RATE_LIMIT_ENABLED=true
RATE_LIMIT_MAX_REQUESTS=1000
SECURITY_STRICT_MODE=false
```

## Best Practices

### 1. Never Commit Secrets

- Use `.env` files for local development (already in `.gitignore`)
- Never commit real credentials, even in examples
- Use placeholders in `.env.example`

### 2. Use Environment Variables

All secrets should be provided via environment variables:

```bash
# Good - using environment variable
export JWT_SECRET="your-secure-secret-key"

# Bad - hardcoding in code
const jwtSecret = "my-secret-key"; // Never do this!
```

### 3. Validate Secret Format

Estuary validates secrets at startup:

- Checks for placeholder values (e.g., "changeme", "xxx", "example")
- Ensures JWT_SECRET is provided when authentication is enabled
- Ensures required secrets are present when features are enabled

### 4. Secret Rotation

When rotating secrets:

1. **Add the new secret** alongside the old one
2. **Deploy the application** with support for both
3. **Update all clients** to use the new secret
4. **Remove the old secret** after confirming all clients are updated

For JWT secret rotation:
- Generate a new strong secret
- Update JWT_SECRET in your deployment
- Restart the application
- Issue new tokens with the new secret

### 5. Development vs Production

Development defaults are intentionally permissive:

```bash
# Development
APP_PROFILE=development
AUTH_ENABLED=false
SECURITY_STRICT_MODE=false

# Production
APP_PROFILE=production
AUTH_ENABLED=true
SECURITY_STRICT_MODE=true
```

## Deployment Environments

### Docker

Use Docker secrets or environment variables:

```yaml
services:
  estuary:
    image: estuary:latest
    environment:
      - DATABASE_TYPE=convex
    secrets:
      - convex_url
      - auth_jwks_urls

secrets:
  convex_url:
    external: true
  auth_jwks_urls:
    external: true
```

### Kubernetes

Use Kubernetes Secrets:

```yaml
apiVersion: v1
kind: Secret
metadata:
  name: estuary-secrets
type: Opaque
data:
  convex-url: <base64-encoded-url>
  auth-jwks-urls: <base64-encoded-urls>
---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: estuary
spec:
  template:
    spec:
      containers:
      - name: estuary
        envFrom:
        - secretRef:
            name: estuary-secrets
```

### Cloud Providers

#### AWS
- Use AWS Secrets Manager or Parameter Store
- Grant EC2/ECS/Lambda roles access to secrets
- Use IAM policies for fine-grained access control

#### Google Cloud
- Use Google Secret Manager
- Configure service account permissions
- Enable automatic secret rotation

#### Azure
- Use Azure Key Vault
- Configure managed identities
- Set up access policies

## Security Monitoring

### Startup Validation

Estuary performs security checks at startup:

```rust
// Automatic validation in Config::load()
- Validates all configuration values
- Checks for placeholder secrets
- Logs warnings for suspicious values
```

### Runtime Protection

- Secrets are never logged in full
- Error messages don't expose sensitive data
- Configuration is logged with secrets redacted

### Audit Logging

When implementing audit logging:
- Log secret access attempts
- Track configuration changes
- Monitor for suspicious patterns
- Never log the actual secret values

## CI/CD Integration

### GitHub Actions

```yaml
name: Deploy
on:
  push:
    branches: [main]

jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Deploy
        env:
          CONVEX_DEPLOYMENT_URL: ${{ secrets.CONVEX_DEPLOYMENT_URL }}
          JWT_SECRET: ${{ secrets.JWT_SECRET }}
        run: |
          # Deploy with secrets from GitHub Secrets
```

### GitLab CI

```yaml
deploy:
  stage: deploy
  script:
    - deploy.sh
  variables:
    CONVEX_DEPLOYMENT_URL: $CONVEX_DEPLOYMENT_URL
    JWT_SECRET: $JWT_SECRET
```

## Secret Scanning

To prevent accidental secret commits:

1. **Pre-commit hooks** (recommended):
   ```bash
   pip install pre-commit
   pre-commit install
   ```

2. **GitHub secret scanning** is automatically enabled

3. **Manual scanning**:
   ```bash
   # Use tools like gitleaks
   gitleaks detect --source . -v
   ```

## Emergency Response

If a secret is accidentally exposed:

1. **Immediately rotate** the compromised secret
2. **Update all systems** using the secret
3. **Review logs** for unauthorized access
4. **Notify** affected users if necessary
5. **Document** the incident and prevention measures

## Configuration Profiles

Estuary supports different configuration profiles:

```bash
# Development (default)
APP_PROFILE=development

# Staging
APP_PROFILE=staging

# Production
APP_PROFILE=production
```

Each profile can have different security defaults:
- Development: Permissive settings, optional authentication
- Staging: Production-like with additional logging
- Production: Strict security, required authentication

## Future Enhancements

### Planned Features

1. **Automatic Secret Rotation**
   - Scheduled rotation for JWKS
   - Database credential rotation
   - API key lifecycle management

2. **Hardware Security Module (HSM) Support**
   - For storing highly sensitive keys
   - PKCS#11 interface support

3. **Vault Integration**
   - HashiCorp Vault support
   - Dynamic secret generation
   - Automatic lease renewal

## Troubleshooting

### Common Issues

1. **"Configuration validation failed"**
   - Check all required environment variables are set
   - Verify secret formats (URLs, etc.)
   - Look for typos in variable names

2. **"Secret appears to contain a placeholder value"**
   - Replace example values with real secrets
   - This warning prevents production deployment with test values

3. **"JWKS fetch failed"**
   - Verify JWKS URL is accessible
   - Check network connectivity
   - Ensure URL is HTTPS in production

### Debug Mode

For troubleshooting (never in production):
```bash
RUST_LOG=debug cargo run
```

This will show configuration loading details with secrets redacted.
