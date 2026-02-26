---
title: Genesis Initialization
description: Creating the first block and initializing your chain
sidebar:
  order: 2
---

Genesis initialization creates the first block (block #0) and sets up the foundation of your blockchain. This is a one-time process when starting a new network.

## What Genesis Creates

When you initialize genesis, the system creates:

1. **Genesis Block** - Block #0 with empty transactions
2. **Base Currencies** - USD, CAD, BTC, ETH
3. **Sovereign Account** - The network administrator
4. **Core Contracts** - System-level smart contracts
5. **Initial Balances** - Starting funds for the sovereign

## Prerequisites

Before running genesis:

```bash
# Install Tana CLI
npm install -g @tananetwork/tana

# Start infrastructure
docker compose up -d  # PostgreSQL + Redis

# Ensure database is accessible
tana test
```

## Creating a Sovereign User

First, create the sovereign user locally:

```bash
tana new user sovereign --name "Network Sovereign" --role sovereign
```

This generates:
- A keypair stored in `~/.config/tana/users/sovereign.json`
- The public key needed for genesis

## Running Genesis

### Option 1: Environment Variables

Set the sovereign credentials and run genesis:

```bash
# Export sovereign public key from your user config
export SOVEREIGN_PUBLIC_KEY=$(cat ~/.config/tana/users/sovereign.json | jq -r '.publicKey')
export SOVEREIGN_USERNAME="@sovereign"
export SOVEREIGN_DISPLAY_NAME="Network Sovereign"

# Optional: customize initial balances
export SOVEREIGN_INITIAL_USD=1000000000    # 1 billion USD
export SOVEREIGN_INITIAL_BTC=100000        # 100k BTC
export SOVEREIGN_INITIAL_ETH=1000000       # 1M ETH

# Run genesis
DATABASE_URL='postgres://...' tana genesis
```

### Option 2: CLI with Genesis Flag

```bash
tana new chain mynetwork --genesis
```

This combines chain creation and genesis initialization.

## Genesis Output

A successful genesis looks like:

```
🌱 Creating Genesis Block (Block #0)...

✅ Genesis block created!

Block Details:
  Height: 0
  Hash: 3a8f2c...
  Previous Hash: 0000000000000000...
  Timestamp: 2024-11-03T00:00:00Z
  Transactions: 0 (self-contained)
  State Changes: 0

🔒 Cryptographic Verification:
  ✓ Block is self-contained (empty genesis state)
  ✓ Transaction root computed (empty tree)
  ✓ State root computed (empty tree)
  ✓ Block hash is deterministic and verifiable

Initializing base currencies...
✅ Base currencies initialized

👑 Creating sovereign account...
  ✅ Sovereign user created: @sovereign
  📋 User ID: usr_abc123...
  🔑 Public Key: ed25519_...

💰 Allocating sovereign initial balances...
  ✅ 1,000,000,000 USD
  ✅ 100,000 BTC
  ✅ 1,000,000 ETH

📦 Deploying core contracts...
  ✅ transfer
  ✅ mint

✅ Deployed 2 core contract(s)

🎉 Blockchain initialized successfully!
```

## Core Contracts

Genesis deploys contracts from the `contracts/core/` directory. These are system-level contracts owned by the sovereign:

| Contract | Purpose |
|----------|---------|
| `transfer` | Standard token transfers |
| `mint` | Currency minting (sovereign only) |

To add custom core contracts, place `.ts` files in `contracts/core/` before running genesis.

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `SOVEREIGN_PUBLIC_KEY` | Ed25519 public key for sovereign | Required |
| `SOVEREIGN_USERNAME` | Username for sovereign account | `@sovereign` |
| `SOVEREIGN_DISPLAY_NAME` | Display name | `Network Sovereign` |
| `SOVEREIGN_INITIAL_USD` | Starting USD balance | `1000000000` |
| `SOVEREIGN_INITIAL_BTC` | Starting BTC balance | `100000` |
| `SOVEREIGN_INITIAL_ETH` | Starting ETH balance | `1000000` |
| `CORE_CONTRACTS_DIR` | Path to core contracts | `./contracts/core` |

## Verifying Genesis

After genesis, verify the chain is initialized:

```bash
# Check block 0 exists
curl http://localhost:8080/blocks/0

# Check sovereign user
curl http://localhost:8080/users

# Check balances
curl "http://localhost:8080/balances?userId=<sovereign_id>"

# Run health checks
tana test
```

## Resetting Genesis

To start over with a fresh chain:

```bash
# Flush all blockchain data (DESTRUCTIVE!)
DATABASE_URL='postgres://...' tana flush

# Re-run genesis
DATABASE_URL='postgres://...' tana genesis
```

> **Warning:** Flushing deletes all users, transactions, blocks, and contracts. Only do this in development or when intentionally resetting.

## Next Steps

- [Validators](/docs/sovereign/validators/) - Add validator nodes to your network
- [Monitoring](/docs/sovereign/monitoring/) - Set up the dashboard
