# Deployment Guide

This guide covers deploying Estuary to production environments, including best practices, scaling considerations, and monitoring setup.

## Table of Contents

1. [Prerequisites](#prerequisites)
2. [Deployment Options](#deployment-options)
3. [Production Configuration](#production-configuration)
4. [Container Deployment](#container-deployment)
5. [Cloud Platform Deployment](#cloud-platform-deployment)
6. [Scaling Considerations](#scaling-considerations)
7. [Monitoring & Observability](#monitoring--observability)
8. [Security Checklist](#security-checklist)
9. [Troubleshooting](#troubleshooting)

## Prerequisites

Before deploying Estuary to production:

1. **Rust toolchain** (for building from source)
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   rustup toolchain install stable
   ```

2. **Environment configuration**
   - Copy `.env.example` to `.env`
   - Configure all required environment variables
   - Ensure secrets are properly managed (see [Secrets Management Guide](./secrets-management.md))

3. **Database setup**
   - For production, consider using Convex or another persistent database
   - In-memory database is suitable only for development/testing

## Deployment Options

### Option 1: Binary Deployment

Build and deploy the binary directly:

```bash
# Build release binary
cargo build --release

# The binary will be at ./target/release/estuary
./target/release/estuary
```

#### Systemd Service (Linux)

Create `/etc/systemd/system/estuary.service`:

```ini
[Unit]
Description=Estuary API Service
After=network.target

[Service]
Type=simple
User=estuary
WorkingDirectory=/opt/estuary
Environment="APP_PROFILE=production"
EnvironmentFile=/opt/estuary/.env
ExecStart=/opt/estuary/estuary
Restart=always
RestartSec=10

# Security hardening
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/opt/estuary/data

[Install]
WantedBy=multi-user.target
```

Enable and start:
```bash
sudo systemctl enable estuary
sudo systemctl start estuary
sudo systemctl status estuary
```

### Option 2: Docker Container

See the [Container Deployment](#container-deployment) section below.

### Option 3: Cloud Platform

See platform-specific guides in the [Cloud Platform Deployment](#cloud-platform-deployment) section.

## Production Configuration

### Essential Environment Variables

```bash
# Server Configuration
APP_PROFILE=production
PORT=3000
RUST_LOG=estuary=info,tower_http=warn

# Database (for Convex)
DATABASE_TYPE=convex
CONVEX_DEPLOYMENT_URL=https://your-deployment.convex.cloud

# Security
SECURITY_STRICT_MODE=true
SECURITY_CSP="default-src 'self'; script-src 'self'; style-src 'self'"

# Rate Limiting
RATE_LIMIT_ENABLED=true
RATE_LIMIT_MAX_REQUESTS=100
RATE_LIMIT_WINDOW_SECONDS=60

# Authentication (if required)
AUTH_ENABLED=true
AUTH_JWKS_URLS=https://your-auth-provider.com/.well-known/jwks.json
AUTH_AUDIENCE=https://api.yourdomain.com
AUTH_ISSUER=https://auth.yourdomain.com/

# Shutdown Grace Period
SHUTDOWN_TIMEOUT_SECONDS=30
```

### Performance Tuning

```bash
# Optimize for your workload
TOKIO_WORKER_THREADS=4  # Default: number of CPU cores

# Connection limits
RUST_LOG=estuary=info,tower_http=warn,tokio=warn
```

## Container Deployment

### Dockerfile

Create a multi-stage Dockerfile for optimal image size:

```dockerfile
# Build stage
FROM rust:1.70-alpine AS builder

# Install build dependencies
RUN apk add --no-cache musl-dev

WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Build dependencies (cached layer)
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -rf src

# Copy source code
COPY . .

# Build application
RUN touch src/main.rs
RUN cargo build --release

# Runtime stage
FROM alpine:3.18

# Install runtime dependencies
RUN apk add --no-cache ca-certificates

# Create non-root user
RUN addgroup -g 1001 estuary && \
    adduser -D -u 1001 -G estuary estuary

WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/estuary /app/estuary

# Copy any static files if needed
# COPY --from=builder /app/static /app/static

# Change ownership
RUN chown -R estuary:estuary /app

USER estuary

EXPOSE 3000

HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD wget --no-verbose --tries=1 --spider http://localhost:3000/health/live || exit 1

CMD ["/app/estuary"]
```

### Docker Compose

For development and simple deployments:

```yaml
version: '3.8'

services:
  estuary:
    build: .
    ports:
      - "3000:3000"
    environment:
      - APP_PROFILE=production
      - DATABASE_TYPE=memory  # Change for production
      - RUST_LOG=estuary=info
    env_file:
      - .env.production
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "wget", "--spider", "-q", "http://localhost:3000/health/live"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 30s

  # Add monitoring stack
  prometheus:
    image: prom/prometheus:latest
    ports:
      - "9090:9090"
    volumes:
      - ./prometheus.yml:/etc/prometheus/prometheus.yml
      - prometheus_data:/prometheus
    command:
      - '--config.file=/etc/prometheus/prometheus.yml'
      - '--storage.tsdb.path=/prometheus'

  grafana:
    image: grafana/grafana:latest
    ports:
      - "3001:3000"
    volumes:
      - grafana_data:/var/lib/grafana
      - ./grafana/provisioning:/etc/grafana/provisioning
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=admin
      - GF_USERS_ALLOW_SIGN_UP=false

volumes:
  prometheus_data:
  grafana_data:
```

### Kubernetes Deployment

Create Kubernetes manifests:

`deployment.yaml`:
```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: estuary
  labels:
    app: estuary
spec:
  replicas: 3
  selector:
    matchLabels:
      app: estuary
  template:
    metadata:
      labels:
        app: estuary
    spec:
      containers:
      - name: estuary
        image: your-registry/estuary:latest
        ports:
        - containerPort: 3000
        env:
        - name: APP_PROFILE
          value: "production"
        - name: PORT
          value: "3000"
        envFrom:
        - configMapRef:
            name: estuary-config
        - secretRef:
            name: estuary-secrets
        resources:
          requests:
            memory: "128Mi"
            cpu: "100m"
          limits:
            memory: "512Mi"
            cpu: "500m"
        livenessProbe:
          httpGet:
            path: /health/live
            port: 3000
          initialDelaySeconds: 10
          periodSeconds: 30
        readinessProbe:
          httpGet:
            path: /health/ready
            port: 3000
          initialDelaySeconds: 5
          periodSeconds: 10
---
apiVersion: v1
kind: Service
metadata:
  name: estuary
spec:
  selector:
    app: estuary
  ports:
  - port: 80
    targetPort: 3000
  type: LoadBalancer
---
apiVersion: v1
kind: ConfigMap
metadata:
  name: estuary-config
data:
  DATABASE_TYPE: "convex"
  RATE_LIMIT_ENABLED: "true"
  RATE_LIMIT_MAX_REQUESTS: "100"
  SECURITY_STRICT_MODE: "true"
---
apiVersion: v1
kind: Secret
metadata:
  name: estuary-secrets
type: Opaque
stringData:
  CONVEX_DEPLOYMENT_URL: "https://your-deployment.convex.cloud"
  AUTH_JWKS_URLS: "https://your-auth.com/.well-known/jwks.json"
```

Apply with:
```bash
kubectl apply -f deployment.yaml
```

## Cloud Platform Deployment

### AWS ECS/Fargate

1. **Build and push image to ECR**:
   ```bash
   aws ecr get-login-password --region us-east-1 | docker login --username AWS --password-stdin $ECR_REGISTRY
   docker build -t estuary .
   docker tag estuary:latest $ECR_REGISTRY/estuary:latest
   docker push $ECR_REGISTRY/estuary:latest
   ```

2. **Task Definition** (`task-definition.json`):
   ```json
   {
     "family": "estuary",
     "networkMode": "awsvpc",
     "requiresCompatibilities": ["FARGATE"],
     "cpu": "256",
     "memory": "512",
     "containerDefinitions": [{
       "name": "estuary",
       "image": "${ECR_REGISTRY}/estuary:latest",
       "portMappings": [{
         "containerPort": 3000,
         "protocol": "tcp"
       }],
       "environment": [
         {"name": "APP_PROFILE", "value": "production"},
         {"name": "PORT", "value": "3000"}
       ],
       "secrets": [
         {
           "name": "CONVEX_DEPLOYMENT_URL",
           "valueFrom": "arn:aws:secretsmanager:region:account:secret:estuary/convex-url"
         }
       ],
       "logConfiguration": {
         "logDriver": "awslogs",
         "options": {
           "awslogs-group": "/ecs/estuary",
           "awslogs-region": "us-east-1",
           "awslogs-stream-prefix": "ecs"
         }
       },
       "healthCheck": {
         "command": ["CMD-SHELL", "wget --spider -q http://localhost:3000/health/live || exit 1"],
         "interval": 30,
         "timeout": 5,
         "retries": 3
       }
     }]
   }
   ```

### Google Cloud Run

Deploy directly from source:

```bash
# Build and deploy
gcloud run deploy estuary \
  --source . \
  --platform managed \
  --region us-central1 \
  --allow-unauthenticated \
  --set-env-vars="APP_PROFILE=production,DATABASE_TYPE=convex" \
  --set-secrets="CONVEX_DEPLOYMENT_URL=estuary-convex-url:latest" \
  --min-instances=1 \
  --max-instances=100 \
  --memory=512Mi \
  --cpu=1
```

### Azure Container Instances

```bash
# Create resource group
az group create --name estuary-rg --location eastus

# Create container
az container create \
  --resource-group estuary-rg \
  --name estuary \
  --image your-registry.azurecr.io/estuary:latest \
  --dns-name-label estuary-api \
  --ports 3000 \
  --environment-variables \
    APP_PROFILE=production \
    DATABASE_TYPE=convex \
  --secure-environment-variables \
    CONVEX_DEPLOYMENT_URL=$CONVEX_URL
```

## Scaling Considerations

### Horizontal Scaling

Estuary is designed to scale horizontally. Key considerations:

1. **Stateless Design**: The application maintains no local state
2. **Database Scaling**:
   - In-memory database doesn't support clustering
   - Use Convex or another distributed database for production
3. **Load Balancing**: Use any standard load balancer (nginx, HAProxy, cloud LB)

### Load Balancer Configuration (nginx)

```nginx
upstream estuary_backend {
    least_conn;
    server estuary1:3000 max_fails=3 fail_timeout=30s;
    server estuary2:3000 max_fails=3 fail_timeout=30s;
    server estuary3:3000 max_fails=3 fail_timeout=30s;
}

server {
    listen 80;
    server_name api.yourdomain.com;

    location / {
        proxy_pass http://estuary_backend;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_cache_bypass $http_upgrade;

        # Timeouts
        proxy_connect_timeout 60s;
        proxy_send_timeout 60s;
        proxy_read_timeout 60s;
    }

    location /health {
        proxy_pass http://estuary_backend/health;
        access_log off;
    }
}
```

### Auto-scaling Configuration

#### Kubernetes HPA (Horizontal Pod Autoscaler)

```yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: estuary-hpa
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: estuary
  minReplicas: 2
  maxReplicas: 10
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 70
  - type: Resource
    resource:
      name: memory
      target:
        type: Utilization
        averageUtilization: 80
```

#### AWS Auto Scaling

For ECS services:
```json
{
  "ServiceName": "estuary",
  "TargetValue": 70.0,
  "PredefinedMetricType": "ECSServiceAverageCPUUtilization",
  "ScaleOutCooldown": 60,
  "ScaleInCooldown": 300,
  "MinCapacity": 2,
  "MaxCapacity": 10
}
```

## Monitoring & Observability

### Prometheus Configuration

Create `prometheus.yml`:

```yaml
global:
  scrape_interval: 15s
  evaluation_interval: 15s

scrape_configs:
  - job_name: 'estuary'
    static_configs:
      - targets: ['estuary:3000']
    metrics_path: '/metrics'
```

### Grafana Dashboard

Import or create dashboards for:

1. **HTTP Metrics**:
   - Request rate by endpoint
   - Response time percentiles (p50, p95, p99)
   - Error rate by status code
   - Active connections

2. **Business Metrics**:
   - Items created/updated/deleted per minute
   - Database query performance
   - Cache hit rates

3. **System Metrics**:
   - CPU usage
   - Memory usage
   - Disk I/O
   - Network traffic

### Example Grafana Query

```promql
# Request rate
rate(http_requests_total[5m])

# 95th percentile response time
histogram_quantile(0.95, rate(http_request_duration_seconds_bucket[5m]))

# Error rate
rate(http_requests_total{status=~"5.."}[5m])

# Database query duration
histogram_quantile(0.95, rate(database_query_duration_seconds_bucket[5m]))
```

### Logging

#### Structured Logging with Vector

`vector.toml`:
```toml
[sources.estuary_logs]
type = "docker_logs"
include_images = ["estuary"]

[transforms.parse_logs]
type = "remap"
inputs = ["estuary_logs"]
source = '''
. = parse_json!(.message)
.timestamp = to_timestamp!(.timestamp)
'''

[sinks.elasticsearch]
type = "elasticsearch"
inputs = ["parse_logs"]
endpoint = "http://elasticsearch:9200"
index = "estuary-%Y.%m.%d"
```

#### CloudWatch Logs (AWS)

```json
{
  "logConfiguration": {
    "logDriver": "awslogs",
    "options": {
      "awslogs-group": "/ecs/estuary",
      "awslogs-region": "us-east-1",
      "awslogs-stream-prefix": "ecs"
    }
  }
}
```

### Alerting Rules

Example Prometheus alerting rules:

```yaml
groups:
  - name: estuary
    rules:
      - alert: HighErrorRate
        expr: rate(http_requests_total{status=~"5.."}[5m]) > 0.05
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High error rate detected"
          description: "Error rate is {{ $value }} errors per second"

      - alert: HighResponseTime
        expr: histogram_quantile(0.95, rate(http_request_duration_seconds_bucket[5m])) > 1
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High response time"
          description: "95th percentile response time is {{ $value }} seconds"

      - alert: DatabaseDown
        expr: up{job="estuary"} == 0
        for: 1m
        labels:
          severity: critical
        annotations:
          summary: "Estuary instance down"
          description: "Estuary instance {{ $labels.instance }} is down"
```

## Security Checklist

Before going to production:

- [ ] Set `APP_PROFILE=production`
- [ ] Enable `SECURITY_STRICT_MODE=true`
- [ ] Configure proper CSP headers
- [ ] Enable authentication if required (`AUTH_ENABLED=true`)
- [ ] Use HTTPS (TLS termination at load balancer)
- [ ] Implement proper secret management
- [ ] Enable rate limiting with appropriate limits
- [ ] Review and update CORS settings
- [ ] Disable debug logging (`RUST_LOG=info`)
- [ ] Set up monitoring and alerting
- [ ] Configure automated backups (if using persistent storage)
- [ ] Implement DDoS protection (CloudFlare, AWS Shield, etc.)
- [ ] Regular security updates and dependency scanning

## Troubleshooting

### Common Issues

1. **High Memory Usage**
   ```bash
   # Check memory metrics
   curl http://localhost:3000/metrics | grep memory

   # Adjust TOKIO_WORKER_THREADS if needed
   ```

2. **Database Connection Issues**
   ```bash
   # Check health endpoint
   curl http://localhost:3000/health/ready

   # Verify database URL and credentials
   ```

3. **Rate Limiting Too Restrictive**
   ```bash
   # Temporarily increase limits
   RATE_LIMIT_MAX_REQUESTS=1000
   ```

4. **JWT Validation Failures**
   ```bash
   # Check JWKS endpoint accessibility
   curl $AUTH_JWKS_URLS

   # Verify audience and issuer match
   ```

### Debug Mode

For troubleshooting production issues:

```bash
# Enable debug logging temporarily
RUST_LOG=estuary=debug,tower_http=debug

# Enable backtrace for panics
RUST_BACKTRACE=1
```

### Performance Profiling

1. **CPU Profiling with perf**:
   ```bash
   perf record -g ./estuary
   perf report
   ```

2. **Memory Profiling**:
   ```bash
   # Use valgrind
   valgrind --leak-check=full ./estuary

   # Or use heaptrack
   heaptrack ./estuary
   ```

3. **Flame Graphs**:
   ```bash
   cargo install flamegraph
   cargo flamegraph --release
   ```

## Maintenance

### Rolling Updates

1. **Kubernetes**:
   ```bash
   kubectl set image deployment/estuary estuary=your-registry/estuary:new-tag
   kubectl rollout status deployment/estuary
   ```

2. **Docker Swarm**:
   ```bash
   docker service update --image your-registry/estuary:new-tag estuary
   ```

3. **Manual**:
   - Deploy new instances
   - Health check new instances
   - Gradually shift traffic
   - Remove old instances

### Backup Strategies

For Convex database:
- Use Convex's built-in backup features
- Schedule regular exports
- Test restore procedures

### Monitoring Checklist

Regular monitoring tasks:
- [ ] Review error logs daily
- [ ] Check metrics dashboards
- [ ] Verify backup completion
- [ ] Review security alerts
- [ ] Check certificate expiration
- [ ] Monitor disk usage
- [ ] Review rate limit violations

## Support

For deployment issues:
1. Check the [troubleshooting](#troubleshooting) section
2. Review application logs
3. Check metrics and monitoring
4. Open an issue on GitHub with:
   - Deployment method
   - Error messages
   - Environment configuration (redacted)
   - Steps to reproduce
