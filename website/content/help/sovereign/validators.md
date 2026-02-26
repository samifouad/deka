---
title: Validators
description: Setting up multi-validator consensus
sidebar:
  order: 3
---

Validators are nodes that participate in consensus to produce blocks. A multi-validator network provides decentralization and fault tolerance.

## Single vs Multi-Validator

| Setup | Use Case | Fault Tolerance |
|-------|----------|-----------------|
| **Single validator** | Development, testing | None |
| **3 validators** | Small production | Tolerates 1 failure |
| **5 validators** | Production | Tolerates 2 failures |
| **7+ validators** | High availability | Tolerates 3+ failures |

## Creating a Validator

On each validator node:

```bash
# Install Tana CLI
npm install -g @tananetwork/tana

# Create validator identity
tana new validator --port 9000 --http-port 9001
```

Output:

```
🔧 Creating validator node...

✅ Validator initialized: val_abc12345
📁 Config saved to: ~/.config/tana/validator.json
🔑 Public key: ed25519_...
🌐 WebSocket URL: ws://localhost:9000
🌐 HTTP API: http://localhost:9001
```

## Validator Configuration

The validator config is stored at `~/.config/tana/validator.json`:

```json
{
  "validatorId": "val_abc12345",
  "publicKey": "ed25519_...",
  "privateKey": "ed25519_...",
  "wsPort": 9000,
  "httpPort": 9001,
  "wsUrl": "ws://localhost:9000",
  "peers": [],
  "createdAt": "2024-11-24T00:00:00Z"
}
```

## Multi-Validator Setup

### Step 1: First Validator (Leader)

On the first machine:

```bash
tana new validator --port 9000 --http-port 9001
```

Note the WebSocket URL: `ws://validator1.example.com:9000`

### Step 2: Additional Validators

On each additional machine, specify the first validator as a peer:

```bash
# Validator 2
tana new validator --port 9000 --http-port 9001 \
  --peers ws://validator1.example.com:9000

# Validator 3
tana new validator --port 9000 --http-port 9001 \
  --peers ws://validator1.example.com:9000,ws://validator2.example.com:9000
```

### Step 3: Start Services

On each validator node:

```bash
# Start infrastructure
docker compose up -d

# Start consensus service
DATABASE_URL='postgres://...' REDIS_URL='redis://...' tana consensus
```

## Network Topology

Validators communicate via WebSocket for P2P consensus:

```
┌─────────────┐       ┌─────────────┐       ┌─────────────┐
│ Validator 1 │◄─────►│ Validator 2 │◄─────►│ Validator 3 │
│   :9000     │       │   :9000     │       │   :9000     │
└──────┬──────┘       └──────┬──────┘       └──────┬──────┘
       │                     │                     │
       └─────────────────────┼─────────────────────┘
                             │
                       ┌─────▼─────┐
                       │   Redis   │
                       │  (shared) │
                       └───────────┘
```

## Port Allocations

| Port | Service | Description |
|------|---------|-------------|
| 9000 | Consensus P2P | WebSocket validator-to-validator |
| 9001 | Consensus API | HTTP health checks |
| 9010 | Validator 2 P2P | (for local multi-validator testing) |
| 9011 | Validator 2 API | |
| 9020 | Validator 3 P2P | |
| 9021 | Validator 3 API | |

## Shared Infrastructure

All validators share:

- **Redis** - Transaction queue (must be network-accessible)
- **Genesis hash** - Same genesis block

Each validator has:

- **PostgreSQL** - Own database (state replicated via consensus)
- **Keypair** - Unique validator identity

## Local Multi-Validator Testing

For testing multiple validators on one machine:

```bash
# Terminal 1 - Validator 1
tana new validator --port 9000 --http-port 9001
DATABASE_URL='...' tana consensus

# Terminal 2 - Validator 2
tana new validator --port 9010 --http-port 9011 --peers ws://localhost:9000
DATABASE_URL='...' tana consensus

# Terminal 3 - Validator 3
tana new validator --port 9020 --http-port 9021 --peers ws://localhost:9000,ws://localhost:9010
DATABASE_URL='...' tana consensus
```

## Health Checks

Check validator status:

```bash
# Consensus service health
curl http://localhost:9001/health

# Check connected peers
curl http://localhost:9001/peers

# Block production status
curl http://localhost:9001/status
```

## Mesh Registration

Validators auto-register with the mesh service via heartbeats:

```bash
# Start mesh service
tana mesh

# View registered validators
curl http://localhost:8190/validators
```

## Production Considerations

### Network Security

- Use TLS for WebSocket connections in production
- Configure firewall rules for validator ports
- Use private network for validator-to-validator traffic

### High Availability

- Deploy validators across multiple regions/zones
- Use managed PostgreSQL for reliability
- Monitor validator health with alerting

### Validator Keys

- Store private keys securely (consider HSM for production)
- Back up validator configs
- Rotate keys periodically

## Troubleshooting

### "Connection refused" to peers

- Ensure firewall allows WebSocket port (9000)
- Verify peer URL is correct
- Check if peer validator is running

### Block production stopped

- Check Redis connectivity
- Verify at least one validator is healthy
- Check consensus logs for errors

### State divergence

- Validators must start from same genesis
- Check for network partitions
- May need to resync from a healthy validator

## Next Steps

- [Monitoring](/docs/sovereign/monitoring/) - Dashboard and observability
- [Deployment](/docs/sovereign/deployment/) - Production deployment guide
