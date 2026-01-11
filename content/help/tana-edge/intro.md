---
title: Edge Server
description: Deploy the HTTP server for off-chain contract execution
sidebar:
  order: 1
---

The Edge server (`tana-edge`) serves HTTP endpoints for your smart contracts. It handles `get()` and `post()` functions with millisecond latency while the blockchain handles consensus operations.

## Why Edge?

| Operation | Blockchain | Edge |
|-----------|------------|------|
| `init()`, `contract()` | ✅ Consensus, state changes | - |
| `get()`, `post()` | - | ✅ Fast HTTP, read-only |

This hybrid model gives you:
- **Speed** - HTTP requests in <100ms
- **No gas fees** - Read operations are free
- **Scalability** - Stateless, horizontally scalable

## Installation

```bash
npm install -g @tananetwork/tana
```

The `tana-edge` binary is included with the CLI package.

## Starting Edge

```bash
# Start edge server
tana-edge --port 8082 --contracts ./contracts

# Or with environment variables
DATABASE_URL='postgres://...' tana-edge
```

## Configuration

| Option | Default | Description |
|--------|---------|-------------|
| `--port` | `8082` | HTTP port |
| `--contracts` | `./contracts` | Contracts directory |
| `--cors` | `*` | CORS origin |

**Environment Variables:**

```bash
DATABASE_URL='postgres://...'  # For blockchain queries
EDGE_PORT=8082
CONTRACTS_DIR=./contracts
```

## Contract Routing

Contracts are routed by path:

```
GET  http://localhost:8082/mycontract  → contracts/mycontract get() function
POST http://localhost:8082/mycontract  → contracts/mycontract post() function
```

In production with subdomain routing:

```
GET  https://mycontract.tana.network  → mycontract get()
POST https://mycontract.tana.network  → mycontract post()
```

## Deploying Contracts

Contracts deployed via `tana deploy contract` are automatically available on edge:

```bash
# Deploy contract
tana deploy contract ./myapp.ts

# Test on edge
curl http://localhost:8082/myapp
```

## Production Setup

### Systemd Service

```ini
# /etc/systemd/system/tana-edge.service
[Unit]
Description=Tana Edge Server
After=network.target

[Service]
Type=simple
User=tana
Environment=DATABASE_URL=postgres://...
ExecStart=/usr/local/bin/tana-edge --port 8082
Restart=always

[Install]
WantedBy=multi-user.target
```

```bash
sudo systemctl enable tana-edge
sudo systemctl start tana-edge
```

### Nginx Reverse Proxy

```nginx
# Subdomain routing
server {
    listen 443 ssl;
    server_name *.tana.network;

    location / {
        proxy_pass http://localhost:8082;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }
}
```

### Docker

```bash
docker run -d \
  -p 8082:8082 \
  -e DATABASE_URL='postgres://...' \
  tananetwork/edge
```

## Health Checks

```bash
# Health endpoint
curl http://localhost:8082/health

# Response
{"status": "ok", "version": "0.1.0"}
```

## Performance

- **Latency**: <100ms (fresh V8 isolate per request)
- **Throughput**: 1000+ requests/second per instance
- **Memory**: Isolated per-request
- **Scaling**: Horizontal (stateless)

## Troubleshooting

### "Contract not found"

- Verify contract is deployed: `curl http://localhost:8080/contracts`
- Check contracts directory path
- Ensure contract has `get()` or `post()` function

### Slow responses

- Check database connectivity
- Monitor V8 isolate creation time
- Consider connection pooling for PostgreSQL

### CORS errors

- Configure `--cors` flag or `CORS_ORIGIN` env var
- For development: `--cors '*'`

## Next Steps

- [Examples](/docs/tana-edge/examples/) - Contract code samples
- [Monitoring](/docs/sovereign/monitoring/) - Health checks and metrics
