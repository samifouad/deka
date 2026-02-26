---
title: Sovereign Overview
description: Run your own Tana blockchain network
sidebar:
  order: 1
---

Tana is designed to be self-hosted. Anyone can become a **sovereign** - running their own blockchain network for a private organization, public testnet, or production deployment.

## Who This Is For

This section is for **sovereigns** (network operators) who want to:

- Run a single-node Tana instance
- Set up a multi-validator network
- Deploy to cloud infrastructure
- Monitor and manage their chain

If you just want to use an existing Tana network, see the [Quick Start](/docs/guides/quickstart/) guide instead.

## Requirements

### Single Node (Development/Testing)

- 2 CPU cores
- 4 GB RAM
- 20 GB SSD
- Node.js 18+ or Bun 1.0+
- PostgreSQL 15+
- Redis 7+

### Production (Multi-Validator)

- 4+ CPU cores per validator
- 8+ GB RAM per validator
- 100+ GB SSD (NVMe recommended)
- Reliable network connectivity between validators

## Installation

```bash
npm install -g @tananetwork/tana
```

Verify installation:

```bash
tana --version
```

## Quick Overview

### 1. Initialize Your Chain

```bash
# Create a new blockchain with genesis block
tana new chain mynetwork --genesis
```

The `--genesis` flag initializes the chain with:
- Genesis block (block 0)
- Core system contracts
- Default currencies (USD, CAD)

### 2. Create the Sovereign User

The first user on a chain is the **sovereign** - they have special privileges for network management.

```bash
tana new user admin --name "Network Admin" --role sovereign
tana deploy user admin
```

### 3. Start Services

Start the required infrastructure and Tana services:

```bash
# Start infrastructure (PostgreSQL, Redis)
docker compose up -d

# Start ledger service
DATABASE_URL='postgres://...' tana ledger

# Start consensus (for validators)
DATABASE_URL='postgres://...' tana consensus
```

### 4. Add Validators (Multi-Node)

For a decentralized network, add additional validator nodes:

```bash
# On validator node 2
tana new validator --port 9010 --http-port 9011 --peers ws://validator1:9000
```

## Network Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                      Your Network                           │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│   ┌───────────┐    ┌───────────┐    ┌───────────┐         │
│   │Validator 1│◄──►│Validator 2│◄──►│Validator 3│         │
│   │  (Leader) │    │           │    │           │         │
│   └─────┬─────┘    └─────┬─────┘    └─────┬─────┘         │
│         │                │                │                │
│         └────────────────┼────────────────┘                │
│                          │                                 │
│                    ┌─────▼─────┐                           │
│                    │   Redis   │  (Shared tx queue)        │
│                    └───────────┘                           │
│                                                             │
├─────────────────────────────────────────────────────────────┤
│   Public API                                                │
│   ┌───────────┐    ┌───────────┐    ┌───────────┐         │
│   │  Ledger   │    │   Edge    │    │ Identity  │         │
│   │  :8080    │    │   :8082   │    │   :8090   │         │
│   └───────────┘    └───────────┘    └───────────┘         │
└─────────────────────────────────────────────────────────────┘
```

## What's Next

- [Starting Docker](/docs/sovereign/docker/) - Bring up all services
- [Genesis Initialization](/docs/sovereign/genesis/) - Understand the genesis process
- [Validators](/docs/sovereign/validators/) - Configure multi-validator consensus
- [Monitoring](/docs/sovereign/monitoring/) - Dashboard and health checks
