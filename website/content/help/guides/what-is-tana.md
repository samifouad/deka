---
title: What is Tana?
description: Understanding Tana - a user-owned blockchain platform
sidebar:
  order: 1
---

Tana is a blockchain platform designed to be **owned and operated by you**. Unlike traditional blockchains that require specialized knowledge and expensive infrastructure, Tana lets anyone run their own blockchain network.

## The Problem with Traditional Blockchains

Most blockchains today are:

- **Complex** - Require learning new programming languages (Solidity, Rust, Move)
- **Expensive** - High gas fees for every operation
- **Centralized in practice** - A few large validators control most networks
- **One-size-fits-all** - You use their chain, their rules, their token

## How Tana is Different

### Write Contracts in TypeScript

No new language to learn. If you know JavaScript or TypeScript, you can write smart contracts:

```typescript
import { kv } from 'tana:kv'

export async function contract() {
  const count = await kv.get('visitors') || '0'
  await kv.put('visitors', String(parseInt(count) + 1))
  return { visitors: parseInt(count) + 1 }
}
```

### No Native Token Required

Tana doesn't force you to use a specific cryptocurrency. Create and manage whatever currencies make sense for your use case - USD, loyalty points, in-game gold, or nothing at all.

### Run Your Own Network

You can:
- **Join** an existing Tana network as a user
- **Run** your own single-node instance
- **Launch** a multi-validator network with others

### Hybrid On-Chain/Off-Chain

Not everything needs to be on the blockchain. Tana separates:
- **Critical operations** (transfers, state changes) → Blockchain consensus
- **Read operations** (queries, APIs) → Fast HTTP servers

## Key Concepts

### Users

Every participant has a **user account** with:
- A unique username
- An Ed25519 keypair for signing transactions
- Balances in multiple currencies

### Contracts

Smart contracts are TypeScript files with up to 4 functions:
- `init()` - Runs once at deployment
- `contract()` - Runs via blockchain transactions
- `get()` - Handles HTTP GET requests
- `post()` - Handles HTTP POST requests

### Validators

Validators are nodes that:
- Receive transactions from users
- Reach consensus on transaction ordering
- Produce new blocks
- Maintain the blockchain state

### Sovereigns

A **sovereign** is someone who runs their own Tana network. They control:
- Who can become a validator
- Initial currency allocations
- Core system contracts

## Use Cases

### Private Organizations

Run an internal blockchain for:
- Asset tracking and provenance
- Internal currencies and rewards
- Audit trails and compliance

### Developer Platforms

Build applications with:
- User accounts and authentication
- Multi-currency payments
- Programmable business logic

### Community Networks

Launch a blockchain for:
- DAOs and governance
- Community currencies
- Shared infrastructure

## Getting Started

- **As a User** → [Quick Start](/docs/guides/quickstart/) - Start using Tana
- **As a Sovereign** → [Sovereign Guide](/docs/sovereign/intro/) - Run your own network
- **As a Developer** → [Developer Setup](/docs/contributing/setup/) - Contribute to Tana
