---
title: API Overview
description: REST API reference for Tana Services
sidebar:
  order: 1
---

This reference documents the REST APIs exposed by Tana services. As a **sovereign** running your own network, you'll use these APIs to integrate with your infrastructure, build dashboards, and create custom tooling.

Tana provides two main REST APIs. All endpoints return JSON responses.

## Base URLs

### Ledger Service (Blockchain API)

```
http://localhost:8080
```

Handles all blockchain operations: users, balances, transactions, blocks, and smart contracts.

### Identity Service (Authentication API)

```
http://localhost:8090
```

Handles mobile-first QR code authentication and session management. Private keys remain exclusively on mobile devices for security.

For production deployments, replace with your server's URLs (e.g., `https://api.tana.network` and `https://auth.tana.network`).

## Authentication & Signatures

All state-changing operations (POST, PATCH, DELETE) require cryptographic signatures to prove ownership.

### Transaction Signatures

**Required Fields:**

```json
{
  "type": "transfer",
  "from": "usr_abc123",
  "to": "usr_xyz789",
  "timestamp": 1699459200,
  "nonce": 42,
  "amount": "100",
  "currencyCode": "USD",
  "signature": "ed25519_sig_..."
}
```

**How Signing Works:**

```typescript
import {
  signMessage,
  createTransactionMessage
} from '@tananetwork/crypto'

// 1. Create canonical transaction message
const tx = {
  type: 'transfer',
  from: 'usr_abc123',
  to: 'usr_xyz789',
  timestamp: Date.now(),
  nonce: 42,
  amount: '100',
  currencyCode: 'USD'
}

// 2. Sign with private key (SHA-256 + Ed25519)
const message = createTransactionMessage(tx)
const signature = await signMessage(message, privateKey)

// 3. Send to API with signature
POST /transactions {
  ...tx,
  signature
}
```

**Server Verification:**

The API verifies:
1. Signature is valid for the canonical message
2. Signature matches the public key on file for `from` user
3. Timestamp is within acceptable range (prevents old transactions)
4. Nonce hasn't been used before (prevents replay attacks)

**Implementation Status:**
- ✅ Ed25519 keypair generation - `@tananetwork/crypto`
- ✅ Transaction signing - CLI and mobile
- ✅ Signature verification - All services
- ✅ Nonce-based replay protection

## Response Format

All API responses follow this format:

### Success Response

```json
{
  "data": { ... },
  "status": "success"
}
```

### Error Response

```json
{
  "error": "Error message describing what went wrong",
  "timestamp": "2024-11-07T00:00:00.000Z"
}
```

## HTTP Status Codes

| Code | Meaning |
|------|---------|
| `200` | Success - Request completed successfully |
| `201` | Created - Resource created successfully |
| `400` | Bad Request - Invalid request parameters |
| `404` | Not Found - Resource doesn't exist |
| `500` | Internal Server Error - Something went wrong on the server |

## Quick Start

### Health Check

```bash
curl http://localhost:8080/health
```

```json
{
  "status": "ok"
}
```

### Get Service Info

```bash
curl http://localhost:8080/
```

```json
{
  "service": "tana-ledger",
  "version": "0.1.0",
  "status": "healthy",
  "timestamp": "2024-11-07T00:00:00.000Z"
}
```

## API Categories

### Identity & Authentication (Port 8090)
Mobile-first QR code authentication with hardware-backed security.

- **POST /auth/session/create** - Create QR code login session
- **GET /auth/session/:id/events** - SSE stream for real-time updates
- **GET /auth/session/:id** - Get session details (mobile scans QR)
- **POST /auth/session/:id/approve** - Approve login with Ed25519 signature
- **POST /auth/session/:id/reject** - Reject login attempt
- **POST /auth/session/validate** - Validate session token

**Security Model:** Private keys ONLY on mobile devices. Desktop/laptop receives session tokens only.

[View Identity API →](/tana-api/identity/)

---

### Users (Port 8080)
Manage user accounts on the blockchain.

- **POST /users** - Create a new user
- **GET /users** - List all users
- **GET /users/:id** - Get user by ID
- **GET /users/by-username/:username** - Get user by username

[View Users API →](/tana-api/users/)

### Balances (Port 8080)
Query and manage multi-currency balances.

- **GET /balances** - Query balances
- **POST /balances** - Set balance
- **GET /balances/currencies** - List supported currencies
- **POST /balances/currencies/seed** - Seed default currencies

[View Balances API →](/tana-api/balances/)

### Transactions (Port 8080)
Create and query blockchain transactions.

- **POST /transactions** - Create transaction
- **GET /transactions** - List transactions
- **GET /transactions/:id** - Get transaction by ID

[View Transactions API →](/tana-api/transactions/)

### Blocks (Port 8080)
Query blockchain blocks and history.

- **GET /blocks** - List blocks
- **GET /blocks/latest** - Get latest block
- **GET /blocks/:height** - Get block by height

[View Blocks API →](/tana-api/blocks/)

### Contracts (Port 8080)
Deploy and manage smart contracts.

- **POST /contracts** - Deploy contract
- **GET /contracts** - List contracts
- **GET /contracts/:id** - Get contract by ID

[View Contracts API →](/tana-api/contracts/)

## Rate Limiting

Currently, there are no rate limits on the development API. Production deployments should implement rate limiting at the reverse proxy level (e.g., Nginx).

## CORS

The API supports CORS for all origins in development mode:

```
Access-Control-Allow-Origin: *
Access-Control-Allow-Credentials: true
```

## Pagination

List endpoints support pagination via query parameters:

```bash
GET /users?page=1&limit=20
```

Default limit is 100 items per page.

## Examples

### Create User and Check Balance

```bash
# 1. Create user
curl -X POST http://localhost:8080/users \
  -H "Content-Type: application/json" \
  -d '{
    "publicKey": "ed25519_abc123...",
    "username": "@alice",
    "displayName": "Alice Johnson"
  }'

# Response:
# {
#   "userId": "user_xyz789",
#   "transactionId": "tx_abc123",
#   "status": "pending"
# }

# 2. Get balance (after block inclusion)
curl "http://localhost:8080/balances?ownerId=user_xyz789&currencyCode=USD"

# Response:
# {
#   "ownerId": "user_xyz789",
#   "ownerType": "user",
#   "currencyCode": "USD",
#   "amount": "1000.00",
#   "updatedAt": "2024-11-07T00:00:00.000Z"
# }
```

## Next Steps

- [Identity API](/tana-api/identity/) - Mobile QR code authentication
- [Users API](/tana-api/users/) - User account management
- [Balances API](/tana-api/balances/) - Balance queries and updates
- [Transactions API](/tana-api/transactions/) - Transaction creation and queries
- [Blocks API](/tana-api/blocks/) - Blockchain queries
- [Contracts API](/tana-api/contracts/) - Smart contract deployment
- [CLI Reference](/tana-cli/intro/) - Command-line interface
