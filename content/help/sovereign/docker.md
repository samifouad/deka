---
title: Starting Docker
description: Bring up all Tana services with Docker Compose
sidebar:
  order: 2
---

This guide covers how to start the complete Tana backend infrastructure using Docker Compose. All services are containerized and orchestrated with health-check-aware dependency ordering.

## Quick Start

From the `engine/` directory:

```bash
cd engine

# Start everything
docker compose up -d

# Check status
docker compose ps

# View logs
docker compose logs -f
```

That's it. Docker Compose handles the startup order automatically via health checks.

## System Architecture

Tana consists of **10 services** organized into three layers:

```
┌─────────────────────────────────────────────────────────────────────┐
│                         PUBLIC LAYER                                 │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│   ┌──────────────────────────────────────────────────────────────┐  │
│   │                     tana-api (8080)                          │  │
│   │              API Gateway - ONLY PUBLIC PORT                  │  │
│   └──────────────────────────────────────────────────────────────┘  │
│                                                                      │
├─────────────────────────────────────────────────────────────────────┤
│                       INTERNAL SERVICES (850X)                       │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│   ┌─────────────┐  ┌─────────────┐  ┌─────────────┐                │
│   │   ledger    │  │    queue    │  │    mesh     │                │
│   │    8501     │  │    8502     │  │    8503     │                │
│   └─────────────┘  └─────────────┘  └─────────────┘                │
│                                                                      │
│   ┌─────────────┐  ┌─────────────┐  ┌─────────────┐                │
│   │  identity   │  │  consensus  │  │    edge     │                │
│   │    8504     │  │  8505/9000  │  │    8506     │                │
│   └─────────────┘  └─────────────┘  └─────────────┘                │
│                                                                      │
│   ┌─────────────┐                                                   │
│   │     t4      │                                                   │
│   │    8507     │                                                   │
│   └─────────────┘                                                   │
│                                                                      │
├─────────────────────────────────────────────────────────────────────┤
│                        INFRASTRUCTURE                                │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│   ┌─────────────────────────┐  ┌─────────────────────────┐         │
│   │       PostgreSQL        │  │         Redis           │         │
│   │          5432           │  │          6379           │         │
│   └─────────────────────────┘  └─────────────────────────┘         │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## Port Allocation Reference

### Public Layer

| Service | Port | Description |
|---------|------|-------------|
| **api** | 8080 | API Gateway - the only public-facing port |

### Internal Services (850X Range)

| Service | Port | Description |
|---------|------|-------------|
| **ledger** | 8501 | Blockchain state, users, blocks, transactions |
| **queue** | 8502 | Transaction ingestion and Redis queue management |
| **mesh** | 8503 | Network coordination and validator discovery |
| **identity** | 8504 | QR authentication and session management |
| **consensus** | 8505 | Validator coordination HTTP API |
| **consensus** | 9000 | P2P WebSocket for validator communication |
| **edge** | 8506 | HTTP contract execution (get/post handlers) |
| **t4** | 8507 | Content-addressable static asset storage |

### Infrastructure

| Service | Port | Description |
|---------|------|-------------|
| **postgres** | 5432 | PostgreSQL database |
| **redis** | 6379 | Transaction queue backend |

## Service Descriptions

### Infrastructure

**PostgreSQL** - Primary database storing all blockchain state:
- Users and accounts
- Transactions and blocks
- Contract deployments
- Session data

**Redis** - High-performance queue for:
- Pending transactions
- Inter-service communication
- Caching

### Stateless Services

**mesh** (8503) - Network coordinator:
- Validator registration and discovery
- Heartbeat monitoring
- Network topology management

**t4** (8507) - Content storage:
- Content-addressable file storage
- Contract source code
- Static assets

### Stateful Services

**ledger** (8501) - Blockchain state manager:
- User account management
- Balance tracking
- Transaction history
- Block storage and queries

**queue** (8502) - Transaction ingestion:
- Receives signed transactions
- Validates transaction format
- Queues for block inclusion

**identity** (8504) - Authentication service:
- QR code generation for mobile auth
- Session management
- Device linking

**consensus** (8505, 9000) - Validator coordination:
- Block production
- P2P communication between validators
- Vote collection and finalization

**edge** (8506) - Contract HTTP server:
- Executes contract `get()` and `post()` handlers
- Fresh V8 isolate per request
- Read-only contract execution

### API Gateway

**api** (8080) - Public API gateway:
- Routes external requests to internal services
- Single entry point for all API calls
- Handles CORS and rate limiting

## Startup Order

Docker Compose automatically handles dependencies via health checks:

```
1. postgres, redis           (infrastructure, no dependencies)
         ↓
2. mesh, t4                  (stateless, no dependencies)
         ↓
3. queue                     (depends on redis)
         ↓
4. ledger                    (depends on postgres, redis, mesh, queue, t4)
         ↓
5. identity, notifications   (depends on postgres)
         ↓
6. consensus                 (depends on postgres, mesh, ledger)
         ↓
7. edge                      (standalone, can start anytime)
         ↓
8. api                       (depends on all services)
```

## Common Commands

```bash
# Start all services
docker compose up -d

# Start only infrastructure (useful for local development)
docker compose up -d postgres redis

# Check service status
docker compose ps

# View logs for all services
docker compose logs -f

# View logs for specific service
docker compose logs -f ledger

# Restart a single service
docker compose restart ledger

# Stop everything
docker compose down

# Stop and remove volumes (DESTRUCTIVE - deletes all data)
docker compose down -v
```

## Health Checks

All services expose `/health` endpoints. Check overall system health:

```bash
# Check all services
for port in 8501 8502 8503 8504 8505 8506 8507 8080; do
  echo -n "Port $port: "
  curl -s http://localhost:$port/health | jq -r '.status // "error"'
done
```

Or use the CLI:

```bash
tana test
```

## Environment Variables

Key environment variables used by the services:

| Variable | Service | Description |
|----------|---------|-------------|
| `DATABASE_URL` | ledger, identity, notifications, consensus | PostgreSQL connection string |
| `REDIS_URL` | queue, ledger | Redis connection string |
| `MESH_URL` | ledger, consensus | Mesh service URL |
| `VALIDATOR_ID` | consensus | Unique validator identifier |
| `SOVEREIGN_PUBLIC_KEY` | mesh | Sovereign's Ed25519 public key |

## Troubleshooting

### Service won't start

Check dependencies are healthy:

```bash
docker compose ps
docker compose logs <service-name>
```

### Port already in use

Find and kill the process:

```bash
lsof -i :<port>
kill <pid>
```

### Database connection errors

Ensure PostgreSQL is healthy and accepting connections:

```bash
docker compose exec postgres pg_isready -U tana
```

### Clean restart

```bash
docker compose down
docker compose up -d
```

## Next Steps

- [Genesis Initialization](/docs/sovereign/genesis/) - Create the first block
- [Validators](/docs/sovereign/validators/) - Add validator nodes
