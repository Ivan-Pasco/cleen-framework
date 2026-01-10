# Frame Framework Deployment Guide

This guide covers deploying Frame applications to production environments across different platforms.

## Table of Contents

1. [Server Deployment](#server-deployment)
2. [Web Deployment](#web-deployment)
3. [Mobile Deployment](#mobile-deployment)
4. [Database Setup](#database-setup)
5. [Environment Variables](#environment-variables)
6. [Performance Optimization](#performance-optimization)
7. [Monitoring](#monitoring)

## Server Deployment

Deploy Frame applications as HTTP servers on any platform that supports WebAssembly.

### Build for Production

```bash
# Build optimized server bundle
frame build --target=server --release

# Output: target/release/server.wasm
```

### Deploy to Docker

**Dockerfile:**

```dockerfile
FROM rust:1.75 as builder

WORKDIR /app
COPY . .

# Install Frame CLI
RUN cargo install frame-cli

# Build application
RUN frame build --target=server --release

# Production image
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    libssl3 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy WASM binary
COPY --from=builder /app/target/release/server.wasm .
COPY --from=builder /app/static ./static

# Copy Frame runtime
COPY --from=builder /usr/local/cargo/bin/frame-runtime /usr/local/bin/

# Set environment variables
ENV PORT=3000
ENV DATABASE_URL=sqlite:./data/app.db

# Create data directory
RUN mkdir -p /app/data

# Expose port
EXPOSE 3000

# Run application
CMD ["frame-runtime", "server.wasm"]
```

**Build and run:**

```bash
# Build Docker image
docker build -t my-frame-app .

# Run container
docker run -p 3000:3000 \
  -e DATABASE_URL="postgres://user:pass@db:5432/myapp" \
  -e SESSION_SECRET="your-secret-key" \
  -v ./data:/app/data \
  my-frame-app
```

### Deploy to Kubernetes

**deployment.yaml:**

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: frame-app
spec:
  replicas: 3
  selector:
    matchLabels:
      app: frame-app
  template:
    metadata:
      labels:
        app: frame-app
    spec:
      containers:
      - name: frame-app
        image: my-frame-app:latest
        ports:
        - containerPort: 3000
        env:
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: frame-secrets
              key: database-url
        - name: SESSION_SECRET
          valueFrom:
            secretKeyRef:
              name: frame-secrets
              key: session-secret
        resources:
          requests:
            memory: "256Mi"
            cpu: "250m"
          limits:
            memory: "512Mi"
            cpu: "500m"
        livenessProbe:
          httpGet:
            path: /health
            port: 3000
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /health
            port: 3000
          initialDelaySeconds: 5
          periodSeconds: 5

---
apiVersion: v1
kind: Service
metadata:
  name: frame-app
spec:
  selector:
    app: frame-app
  ports:
  - protocol: TCP
    port: 80
    targetPort: 3000
  type: LoadBalancer
```

**Apply:**

```bash
kubectl apply -f deployment.yaml
```

### Deploy to Cloud Platforms

#### AWS Lambda

```bash
# Build for Lambda
frame build --target=lambda --release

# Package for deployment
zip function.zip target/release/lambda.wasm

# Deploy with AWS CLI
aws lambda create-function \
  --function-name my-frame-app \
  --runtime provided.al2 \
  --handler bootstrap \
  --zip-file fileb://function.zip \
  --role arn:aws:iam::ACCOUNT:role/lambda-role
```

#### Google Cloud Run

```bash
# Build Docker image
docker build -t gcr.io/PROJECT/frame-app .

# Push to Container Registry
docker push gcr.io/PROJECT/frame-app

# Deploy to Cloud Run
gcloud run deploy frame-app \
  --image gcr.io/PROJECT/frame-app \
  --platform managed \
  --region us-central1 \
  --allow-unauthenticated
```

#### Azure Container Instances

```bash
# Create resource group
az group create --name frame-app-rg --location eastus

# Create container instance
az container create \
  --resource-group frame-app-rg \
  --name frame-app \
  --image my-frame-app:latest \
  --dns-name-label frame-app \
  --ports 3000 \
  --environment-variables \
    DATABASE_URL="$DATABASE_URL" \
    SESSION_SECRET="$SESSION_SECRET"
```

## Web Deployment

Deploy Frame applications as static websites with WebAssembly.

### Build for Web

```bash
# Build web bundle
frame build --target=web --release

# Output directory: target/web/
```

This creates:
```
target/web/
├── index.html        # Main HTML file
├── app.wasm          # Application WASM
├── frame.js          # Frame runtime
└── static/           # Static assets
```

### Deploy to Static Hosting

#### Netlify

**netlify.toml:**

```toml
[build]
  command = "frame build --target=web --release"
  publish = "target/web"

[[headers]]
  for = "/*.wasm"
  [headers.values]
    Content-Type = "application/wasm"
    Cache-Control = "public, max-age=31536000"

[[headers]]
  for = "/*.js"
  [headers.values]
    Cache-Control = "public, max-age=31536000"
```

Deploy:
```bash
netlify deploy --prod
```

#### Vercel

**vercel.json:**

```json
{
  "buildCommand": "frame build --target=web --release",
  "outputDirectory": "target/web",
  "headers": [
    {
      "source": "/*.wasm",
      "headers": [
        {
          "key": "Content-Type",
          "value": "application/wasm"
        }
      ]
    }
  ]
}
```

Deploy:
```bash
vercel --prod
```

#### AWS S3 + CloudFront

```bash
# Build
frame build --target=web --release

# Sync to S3
aws s3 sync target/web/ s3://my-bucket/ \
  --exclude "*.wasm" \
  --cache-control "public, max-age=31536000"

# Upload WASM files with correct content type
aws s3 sync target/web/ s3://my-bucket/ \
  --exclude "*" \
  --include "*.wasm" \
  --content-type "application/wasm" \
  --cache-control "public, max-age=31536000"

# Invalidate CloudFront cache
aws cloudfront create-invalidation \
  --distribution-id DISTRIBUTION_ID \
  --paths "/*"
```

## Mobile Deployment

Deploy Frame applications as native mobile apps.

### iOS Deployment

```bash
# Initialize iOS project
frame mobile:init ios

# Build for iOS
frame build --target=ios --release

# Open in Xcode
open platforms/ios/MyApp.xcodeproj
```

**In Xcode:**
1. Set signing team
2. Configure bundle identifier
3. Build and archive
4. Upload to App Store Connect

### Android Deployment

```bash
# Initialize Android project
frame mobile:init android

# Build APK
frame build --target=android --release

# Output: platforms/android/app/build/outputs/apk/release/app-release.apk
```

**Sign and publish:**

```bash
# Generate signing key (first time only)
keytool -genkey -v -keystore my-release-key.jks \
  -keyalg RSA -keysize 2048 -validity 10000 \
  -alias my-key-alias

# Sign APK
jarsigner -verbose -sigalg SHA256withRSA \
  -digestalg SHA-256 \
  -keystore my-release-key.jks \
  app-release-unsigned.apk my-key-alias

# Align APK
zipalign -v 4 app-release-unsigned.apk app-release.apk

# Upload to Google Play Console
```

## Database Setup

### PostgreSQL (Recommended for Production)

**1. Install PostgreSQL:**

```bash
# Ubuntu/Debian
sudo apt-get install postgresql

# macOS
brew install postgresql

# Start service
sudo systemctl start postgresql
```

**2. Create database:**

```sql
CREATE DATABASE myapp_production;
CREATE USER myapp WITH PASSWORD 'secure-password';
GRANT ALL PRIVILEGES ON DATABASE myapp_production TO myapp;
```

**3. Configure connection:**

```bash
export DATABASE_URL="postgres://myapp:secure-password@localhost:5432/myapp_production"
```

**4. Run migrations:**

```bash
frame db:migrate
```

### Managed Database Services

#### AWS RDS

```bash
# Create RDS instance
aws rds create-db-instance \
  --db-instance-identifier frame-db \
  --db-instance-class db.t3.micro \
  --engine postgres \
  --master-username admin \
  --master-user-password SECRET \
  --allocated-storage 20

# Get connection string
aws rds describe-db-instances \
  --db-instance-identifier frame-db \
  --query 'DBInstances[0].Endpoint.Address'

# Set DATABASE_URL
export DATABASE_URL="postgres://admin:SECRET@endpoint.rds.amazonaws.com:5432/myapp"
```

#### Google Cloud SQL

```bash
# Create instance
gcloud sql instances create frame-db \
  --database-version=POSTGRES_14 \
  --tier=db-f1-micro \
  --region=us-central1

# Create database
gcloud sql databases create myapp \
  --instance=frame-db

# Set password
gcloud sql users set-password postgres \
  --instance=frame-db \
  --password=SECRET
```

## Environment Variables

### Required Variables

```bash
# Database
DATABASE_URL="postgres://user:pass@host:5432/db"

# Authentication
SESSION_SECRET="random-secret-key-min-32-chars"
JWT_SECRET="another-random-secret-key"

# Environment
NODE_ENV="production"
PORT="3000"
```

### Optional Variables

```bash
# Logging
LOG_LEVEL="info"  # debug, info, warn, error

# CORS
CORS_ORIGIN="https://example.com"

# Rate limiting
RATE_LIMIT_MAX="100"
RATE_LIMIT_WINDOW="900000"  # 15 minutes in ms

# File uploads
MAX_FILE_SIZE="10485760"  # 10MB in bytes

# Email (if using)
SMTP_HOST="smtp.example.com"
SMTP_PORT="587"
SMTP_USER="noreply@example.com"
SMTP_PASSWORD="password"
```

### Managing Secrets

#### Docker

```bash
docker run -p 3000:3000 \
  --env-file .env.production \
  my-frame-app
```

#### Kubernetes

```bash
# Create secret
kubectl create secret generic frame-secrets \
  --from-literal=database-url="$DATABASE_URL" \
  --from-literal=session-secret="$SESSION_SECRET"

# Reference in deployment (see Kubernetes section above)
```

#### AWS Secrets Manager

```bash
# Store secret
aws secretsmanager create-secret \
  --name frame/database-url \
  --secret-string "$DATABASE_URL"

# Retrieve in application startup
aws secretsmanager get-secret-value \
  --secret-id frame/database-url \
  --query SecretString \
  --output text
```

## Performance Optimization

### 1. Enable Compression

**Nginx:**

```nginx
gzip on;
gzip_types text/plain text/css application/json application/javascript application/wasm;
gzip_min_length 1000;
```

**Apache:**

```apache
<IfModule mod_deflate.c>
  AddOutputFilterByType DEFLATE text/html text/plain text/css application/json application/javascript application/wasm
</IfModule>
```

### 2. Set Cache Headers

```nginx
location ~* \.(wasm|js|css)$ {
  expires 1y;
  add_header Cache-Control "public, immutable";
}
```

### 3. Enable HTTP/2

**Nginx:**

```nginx
server {
  listen 443 ssl http2;
  # ... rest of config
}
```

### 4. Configure CDN

Use a CDN like Cloudflare, AWS CloudFront, or Fastly:

```bash
# Example with Cloudflare
# 1. Add site to Cloudflare
# 2. Update DNS to Cloudflare nameservers
# 3. Enable caching rules
# 4. Enable HTTP/3 and Brotli compression
```

### 5. Optimize Database

**Add indexes:**

```sql
-- For frequently queried fields
CREATE INDEX idx_posts_author_id ON posts(author_id);
CREATE INDEX idx_posts_created_at ON posts(created_at DESC);

-- For search
CREATE INDEX idx_posts_title_gin ON posts USING gin(to_tsvector('english', title));
```

**Connection pooling:**

```bash
# Set in DATABASE_URL
DATABASE_URL="postgres://user:pass@host/db?pool_size=20"
```

## Monitoring

### 1. Health Checks

Frame includes built-in health endpoints:

```bash
GET /health        # Basic health check
GET /health/ready  # Readiness probe
GET /health/live   # Liveness probe
GET /metrics       # Prometheus metrics
```

### 2. Logging

Configure structured logging:

```bash
# Set log level
export LOG_LEVEL=info

# JSON format for production
export LOG_FORMAT=json
```

### 3. Application Monitoring

#### Prometheus + Grafana

**prometheus.yml:**

```yaml
scrape_configs:
  - job_name: 'frame-app'
    static_configs:
      - targets: ['localhost:3000']
    metrics_path: '/metrics'
```

#### DataDog

```bash
# Install DataDog agent
DD_API_KEY=<key> bash -c "$(curl -L https://s3.amazonaws.com/dd-agent/scripts/install_script.sh)"

# Configure StatsD
export DD_AGENT_HOST=localhost
export DD_DOGSTATSD_PORT=8125
```

#### New Relic

```bash
# Install New Relic agent
export NEW_RELIC_LICENSE_KEY=<key>
export NEW_RELIC_APP_NAME="Frame App"
```

### 4. Error Tracking

#### Sentry

```bash
# Set Sentry DSN
export SENTRY_DSN="https://...@sentry.io/..."

# Errors are automatically reported
```

## Security Checklist

Before deploying to production:

- [ ] Environment variables are set securely
- [ ] Database credentials are not in source code
- [ ] HTTPS is enforced
- [ ] CORS is properly configured
- [ ] Rate limiting is enabled
- [ ] SQL injection protection (automatic in Frame)
- [ ] XSS protection (automatic HTML escaping)
- [ ] CSRF tokens enabled for forms
- [ ] Secure cookie flags set (httpOnly, secure, sameSite)
- [ ] Dependencies are up to date
- [ ] Secrets are rotated regularly
- [ ] Backups are automated
- [ ] Monitoring and alerting configured

## Scaling

### Horizontal Scaling

Frame applications are stateless and scale horizontally:

**Kubernetes:**

```bash
# Scale to 5 replicas
kubectl scale deployment frame-app --replicas=5

# Auto-scaling
kubectl autoscale deployment frame-app \
  --cpu-percent=70 \
  --min=3 \
  --max=10
```

**AWS ECS:**

```bash
# Update service desired count
aws ecs update-service \
  --cluster frame-cluster \
  --service frame-app \
  --desired-count 5
```

### Load Balancing

**Nginx:**

```nginx
upstream frame_app {
  least_conn;
  server app1.example.com:3000;
  server app2.example.com:3000;
  server app3.example.com:3000;
}

server {
  listen 80;
  server_name example.com;

  location / {
    proxy_pass http://frame_app;
    proxy_set_header Host $host;
    proxy_set_header X-Real-IP $remote_addr;
  }
}
```

### Database Scaling

**Read replicas:**

```bash
# Primary (writes)
PRIMARY_DB="postgres://user:pass@primary:5432/db"

# Replica (reads)
REPLICA_DB="postgres://user:pass@replica:5432/db"
```

**Connection pooling:**

```bash
# Use PgBouncer
DATABASE_URL="postgres://user:pass@pgbouncer:6432/db?pool_size=20"
```

## Troubleshooting

### Application won't start

1. Check environment variables: `env | grep DATABASE_URL`
2. Verify database connection: `frame db:status`
3. Check logs: `docker logs container-name`
4. Verify WASM runtime version

### Performance issues

1. Enable query logging: `LOG_LEVEL=debug`
2. Check database indexes: `EXPLAIN ANALYZE SELECT...`
3. Monitor memory usage: `docker stats`
4. Profile WASM execution

### Database connection errors

1. Verify DATABASE_URL format
2. Check firewall rules
3. Verify database is running
4. Check connection pool settings

## Backup and Recovery

### Database Backups

**Automated backup script:**

```bash
#!/bin/bash
BACKUP_DIR="/backups"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)

# Backup database
pg_dump $DATABASE_URL > "$BACKUP_DIR/backup_$TIMESTAMP.sql"

# Compress
gzip "$BACKUP_DIR/backup_$TIMESTAMP.sql"

# Keep only last 30 days
find $BACKUP_DIR -name "backup_*.sql.gz" -mtime +30 -delete
```

**Restore:**

```bash
gunzip -c backup_20250101_120000.sql.gz | psql $DATABASE_URL
```

### File Backups

```bash
# Backup uploads directory
tar -czf uploads_$(date +%Y%m%d).tar.gz /app/uploads

# Upload to S3
aws s3 cp uploads_20250101.tar.gz s3://backups/
```

---

## Next Steps

- Read [Performance Tuning Guide](PERFORMANCE.md)
- Review [Security Best Practices](SECURITY.md)
- Check [Monitoring Guide](MONITORING.md)

**Your Frame application is now production-ready!** 🚀
