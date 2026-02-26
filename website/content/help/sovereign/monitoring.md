---
title: Monitoring
description: Dashboard and health checks for your network
sidebar:
  order: 4
---

Tana provides built-in monitoring tools to observe your network's health, track transactions, and manage validators.

## Health Checks

### Quick Health Check

```bash
tana test
```

This checks:
- Database connectivity
- Migrations applied
- Service availability
- Block production status

### Service Health Endpoints

Each service exposes a `/health` endpoint:

```bash
# Ledger service
curl http://localhost:8080/health

# Identity service
curl http://localhost:8090/health

# Consensus service
curl http://localhost:9001/health

# Mesh service
curl http://localhost:8190/health

# Edge server
curl http://localhost:8082/health
```

## Unified Dashboard

The mesh service includes a web-based dashboard for monitoring:

### Starting the Dashboard

```bash
# Start mesh service with dashboard
tana mesh
```

Access at: `http://localhost:8190`

### Dashboard Views

The dashboard has four main views:

| View | Description |
|------|-------------|
| **Blocks** | Recent blocks, transactions per block, finalization status |
| **Events** | Real-time event stream from all services |
| **Users** | User accounts, balances, search |
| **Network** | Validator topology, chaos engineering controls |

### Blocks View

Monitor blockchain progress:
- Block height and hash
- Transactions per block
- Producer (validator)
- Finalization timestamps

### Events View

Real-time event streaming from the event-bus:
- Transaction lifecycle events
- Block production events
- Consensus events
- Filter by service, level, or category

### Users View

User management:
- Search users by username or ID
- View balances across currencies
- User role (sovereign, staff, user)
- Transaction history

### Network View

Network topology and operations:
- Validator status (online/offline)
- Peer connections (force-directed graph)
- Chaos engineering controls
- Network health metrics

## Chaos Engineering

The network view includes chaos engineering controls for testing fault tolerance:

### Available Actions

| Action | Description |
|--------|-------------|
| **Kill Validator** | Terminate a validator process |
| **Partition** | Simulate network partition |
| **Slow Network** | Add artificial latency |
| **Restore** | Recover from chaos state |

> **Warning:** Only use chaos controls in test environments!

## Metrics

### Block Production Metrics

```bash
# Latest block
curl http://localhost:8080/blocks/latest

# Block production rate
curl http://localhost:8080/blocks?limit=10
```

### Transaction Metrics

```bash
# Pending transactions
curl http://localhost:8080/transactions/pending

# Transaction throughput
curl http://localhost:8080/transactions/count
```

### Validator Metrics

```bash
# Registered validators
curl http://localhost:8190/validators

# Validator health
curl http://localhost:9001/status
```

## Logging

### Log Levels

Configure logging via environment:

```bash
# Development (verbose)
LOG_LEVEL=debug tana ledger

# Production (errors only)
LOG_LEVEL=error tana ledger
```

### Event-Bus Integration

For structured logging across services, use the event-bus:

```bash
# Query recent events
curl http://localhost:8200/events?limit=100

# Filter by service
curl http://localhost:8200/events?service=ledger

# Filter by level
curl http://localhost:8200/events?level=error
```

## Alerting

For production deployments, integrate with external monitoring:

### Prometheus Metrics

Export metrics for Prometheus scraping:

```bash
# Metrics endpoint (if enabled)
curl http://localhost:8080/metrics
```

### Health Check Integration

Use health endpoints with your monitoring stack:

```yaml
# Example: Docker health check
healthcheck:
  test: ["CMD", "curl", "-f", "http://localhost:8080/health"]
  interval: 30s
  timeout: 10s
  retries: 3
```

## Troubleshooting

### Block Production Stalled

1. Check validator health: `curl http://localhost:9001/health`
2. Check Redis: `redis-cli ping`
3. Check for consensus errors in logs
4. Verify validators can reach each other

### High Transaction Latency

1. Check pending transaction count
2. Verify Redis isn't overloaded
3. Check database connection pool
4. Monitor block production rate

### Service Not Responding

1. Check if process is running: `ps aux | grep tana`
2. Check port availability: `lsof -i :8080`
3. Review service logs
4. Verify database connectivity

## Next Steps

- [Deployment](/docs/sovereign/deployment/) - Production deployment guide
- [API Reference](/docs/tana-api/intro/) - Full API documentation
