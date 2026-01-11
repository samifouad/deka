---
title: Production Deployment
description: Deploy Tana blockchain to production
sidebar:
  order: 4
---

This guide covers deploying the Tana blockchain infrastructure to production.

## Prerequisites

### Server Requirements
- **CPU:** 4+ cores
- **RAM:** 8GB+ minimum, 16GB recommended
- **Storage:** 100GB+ SSD
- **OS:** Ubuntu 22.04 LTS (recommended)
- **Network:** Static IP, ports 80/443/8080 accessible

### Software Requirements
- Docker 24.0+
- Docker Compose 2.0+
- Bun 1.1.38+ (for migrations)
- PostgreSQL 16 (via Docker)
- Redis 7 (via Docker)

### Domain Configuration
Configure DNS records for your domain:
```
A     blockchain.yourdomain.com    → <SERVER_IP>
AAAA  blockchain.yourdomain.com    → <SERVER_IPv6>  (optional)
```

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     yourdomain.com                           │
│                  (Main Website/Playground)                   │
└─────────────────────────────────────────────────────────────┘
                            │
                            ↓ HTTPS
┌─────────────────────────────────────────────────────────────┐
│         blockchain.yourdomain.com (Load Balancer)           │
└─────────────────────────────────────────────────────────────┘
                            │
           ┌────────────────┼────────────────┐
           ↓                ↓                ↓
    ┌──────────┐    ┌──────────┐    ┌──────────┐
    │ Ledger   │    │  Edge    │    │ Runtime  │
    │ Service  │    │ Server   │    │ Service  │
    │  :8080   │    │  :8180   │    │          │
    └──────────┘    └──────────┘    └──────────┘
           │                │
           └────────────────┘
                    ↓
            ┌──────────────┐
            │  PostgreSQL  │
            │     :5432    │
            └──────────────┘
                    │
            ┌──────────────┐
            │    Redis     │
            │     :6379    │
            └──────────────┘
```

## Services Overview

### Ledger Service (Port 8080)
**Purpose:** Blockchain data API - users, balances, transactions

**Key Endpoints:**
- `GET /health` - Health check
- `GET /users` - List all users
- `POST /users` - Create user
- `GET /balances` - Get all balances
- `GET /transactions` - Get all transactions
- `POST /transactions` - Create transaction

### Edge Server (Port 8180)
**Purpose:** Fast HTTP execution of smart contracts

**Features:**
- TypeScript contract execution
- V8 isolate per request
- Cloudflare Workers-style architecture
- Sub-100ms response times

## Installation

### 1. Clone Repository
```bash
git clone https://github.com/yourusername/tana.git
cd tana
```

### 2. Create Production Environment File
```bash
cp .env.example .env.production
```

Edit `.env.production`:
```bash
# Database Configuration
POSTGRES_DB=tana
POSTGRES_USER=tana
POSTGRES_PASSWORD=<STRONG_PASSWORD_HERE>

# Ledger URL (for all services)
TANA_LEDGER_URL=https://blockchain.yourdomain.com

# Service Configuration
NODE_ENV=production
LOG_LEVEL=info

# CORS Origins
ALLOWED_ORIGINS=https://yourdomain.com,https://blockchain.yourdomain.com
```

### 3. Build Docker Images
```bash
# Build all services
docker-compose -f docker-compose.prod.yml build

# Or build specific service
docker-compose -f docker-compose.prod.yml build tana-ledger
```

### 4. Initialize Database
```bash
# Start database services
docker-compose -f docker-compose.prod.yml up -d postgres redis

# Wait for postgres to be ready
docker-compose -f docker-compose.prod.yml exec postgres pg_isready

# Run migrations
cd ledger
bun run db:migrate

# Seed default data
curl -X POST http://localhost:8080/balances/currencies/seed
```

### 5. Start Services
```bash
# Start all services
docker-compose -f docker-compose.prod.yml up -d

# Check status
docker-compose -f docker-compose.prod.yml ps

# View logs
docker-compose -f docker-compose.prod.yml logs -f
```

### 6. Verify Deployment
```bash
# Test ledger service
curl http://localhost:8080/health
curl http://localhost:8080/

# Test endpoints
curl http://localhost:8080/users
curl http://localhost:8080/balances
curl http://localhost:8080/transactions
```

## Reverse Proxy Configuration

### Using Nginx

Create `/etc/nginx/sites-available/blockchain.yourdomain.com`:

```nginx
# HTTP -> HTTPS redirect
server {
    listen 80;
    listen [::]:80;
    server_name blockchain.yourdomain.com;

    location / {
        return 301 https://$server_name$request_uri;
    }
}

# HTTPS
server {
    listen 443 ssl http2;
    listen [::]:443 ssl http2;
    server_name blockchain.yourdomain.com;

    # SSL certificates (Let's Encrypt)
    ssl_certificate /etc/letsencrypt/live/blockchain.yourdomain.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/blockchain.yourdomain.com/privkey.pem;
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers HIGH:!aNULL:!MD5;

    # Security headers
    add_header Strict-Transport-Security "max-age=31536000; includeSubDomains" always;
    add_header X-Frame-Options "SAMEORIGIN" always;
    add_header X-Content-Type-Options "nosniff" always;
    add_header X-XSS-Protection "1; mode=block" always;

    # CORS headers
    add_header Access-Control-Allow-Origin "https://yourdomain.com" always;
    add_header Access-Control-Allow-Methods "GET, POST, OPTIONS" always;
    add_header Access-Control-Allow-Headers "Content-Type, Authorization" always;

    # Rate limiting
    limit_req_zone $binary_remote_addr zone=api_limit:10m rate=10r/s;
    limit_req zone=api_limit burst=20 nodelay;

    # Proxy to ledger service
    location / {
        proxy_pass http://localhost:8080;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_cache_bypass $http_upgrade;

        # Timeouts
        proxy_connect_timeout 30s;
        proxy_send_timeout 30s;
        proxy_read_timeout 30s;
    }

    # Health check endpoint (no rate limiting)
    location /health {
        proxy_pass http://localhost:8080/health;
        access_log off;
    }
}
```

Enable and test:
```bash
# Enable site
sudo ln -s /etc/nginx/sites-available/blockchain.yourdomain.com /etc/nginx/sites-enabled/

# Test configuration
sudo nginx -t

# Reload nginx
sudo systemctl reload nginx
```

### SSL Certificate (Let's Encrypt)
```bash
# Install certbot
sudo apt update
sudo apt install certbot python3-certbot-nginx

# Get certificate
sudo certbot --nginx -d blockchain.yourdomain.com

# Auto-renewal is enabled by default
# Test renewal
sudo certbot renew --dry-run
```

## Monitoring & Maintenance

### View Logs
```bash
# All services
docker-compose -f docker-compose.prod.yml logs -f

# Specific service
docker-compose -f docker-compose.prod.yml logs -f tana-ledger

# Last 100 lines
docker-compose -f docker-compose.prod.yml logs --tail=100 tana-ledger
```

### Database Backups
Create a backup script `/usr/local/bin/backup-tana.sh`:

```bash
#!/bin/bash
BACKUP_DIR="/var/backups/tana"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
mkdir -p $BACKUP_DIR

# Backup database
docker-compose -f /path/to/tana/docker-compose.prod.yml exec -T postgres \
  pg_dump -U tana tana | gzip > "$BACKUP_DIR/tana_$TIMESTAMP.sql.gz"

# Keep only last 7 days
find $BACKUP_DIR -name "tana_*.sql.gz" -mtime +7 -delete
```

Add to crontab:
```bash
# Daily backup at 2 AM
0 2 * * * /usr/local/bin/backup-tana.sh
```

### Health Monitoring
```bash
# Check service health
curl https://blockchain.yourdomain.com/health

# Monitor container stats
docker stats

# Check service status
docker-compose -f docker-compose.prod.yml ps
```

### Updates & Upgrades
```bash
# Pull latest code
git pull origin main

# Rebuild images
docker-compose -f docker-compose.prod.yml build

# Run database migrations
cd ledger && bun run db:migrate && cd ..

# Restart services (zero downtime)
docker-compose -f docker-compose.prod.yml up -d --no-deps --build tana-ledger
docker-compose -f docker-compose.prod.yml up -d --no-deps --build tana-edge
```

## Security Checklist

- [ ] Strong PostgreSQL password set
- [ ] SSL certificates installed and valid
- [ ] Firewall configured (UFW or iptables)
- [ ] Rate limiting enabled in Nginx
- [ ] Services running as non-root user
- [ ] Database not exposed to public internet
- [ ] Redis not exposed to public internet
- [ ] CORS properly configured
- [ ] Security headers enabled
- [ ] Regular backups configured
- [ ] Monitoring and alerting set up
- [ ] Secrets stored securely (not in git)
- [ ] `TANA_LEDGER_URL` environment variable set correctly

## Troubleshooting

### Service Won't Start
```bash
# Check logs
docker-compose -f docker-compose.prod.yml logs tana-ledger

# Check container status
docker-compose -f docker-compose.prod.yml ps

# Restart specific service
docker-compose -f docker-compose.prod.yml restart tana-ledger
```

### Database Connection Issues
```bash
# Check postgres is running
docker-compose -f docker-compose.prod.yml ps postgres

# Test connection manually
docker-compose -f docker-compose.prod.yml exec postgres \
  psql -U tana -d tana -c "SELECT version();"
```

### High Memory Usage
```bash
# Check resource usage
docker stats

# Adjust resource limits in docker-compose.prod.yml
services:
  tana-ledger:
    deploy:
      resources:
        limits:
          memory: 512M
```

### CORS Errors
Check `ALLOWED_ORIGINS` in `.env.production` includes all necessary domains:
```bash
ALLOWED_ORIGINS=https://yourdomain.com,https://blockchain.yourdomain.com
```

## Performance Tuning

### PostgreSQL Optimization
Edit `docker-compose.prod.yml` to add PostgreSQL performance settings:

```yaml
postgres:
  command: >
    postgres
    -c shared_buffers=256MB
    -c effective_cache_size=1GB
    -c maintenance_work_mem=128MB
    -c checkpoint_completion_target=0.9
    -c wal_buffers=16MB
    -c default_statistics_target=100
    -c random_page_cost=1.1
    -c effective_io_concurrency=200
    -c work_mem=16MB
    -c min_wal_size=1GB
    -c max_wal_size=4GB
```

## Cost Estimation

### Minimum Setup (2 core VPS)
- **DigitalOcean:** $12-24/month
- **Hetzner:** €5-15/month
- **Linode:** $12-24/month

### Recommended Setup (4 core VPS)
- **DigitalOcean:** $48/month
- **Hetzner:** €20/month
- **AWS Lightsail:** $40/month

### Additional Costs
- Domain: $10-20/year
- Backups: $5-10/month (optional)
- Monitoring: Free (self-hosted) or $20/month

## Next Steps

After successful deployment:
1. Monitor logs for the first 24 hours
2. Set up automated backups
3. Configure uptime monitoring
4. Test all API endpoints from your application
5. Review and optimize performance

## Related Documentation

- [Environment Variables](/contributing/env-vars/) - Configuration options
- [API Reference](/tana-api/intro/) - Ledger API endpoints
- [Edge Server](/tana-edge/intro/) - Edge server documentation
- [Development Setup](/contributing/setup/) - Local development guide
