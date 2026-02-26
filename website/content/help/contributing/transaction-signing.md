---
title: Transaction Signing
description: How to sign and verify transactions with Ed25519
sidebar:
  order: 4
---

All transactions on Tana require Ed25519 cryptographic signatures to ensure authenticity and prevent unauthorized actions.

## Overview

Every transaction must be signed by the private key of the account initiating the transaction. This prevents:

- **Impersonation** - No one can create transactions on your behalf
- **Replay attacks** - Transactions cannot be reused (thanks to nonces)
- **Tampering** - Transaction data cannot be modified after signing

## Signature Requirements

Every transaction (except user creation) must include:

1. **Signature** - Ed25519 signature (hex string with `ed25519_` prefix)
2. **Timestamp** - Unix timestamp in milliseconds (must be within 5 minutes of current time)
3. **Nonce** - Sequential number that increments with each transaction

## How Signing Works

### 1. Get Your Current Nonce

Before creating a transaction, fetch your current nonce:

```bash
# Get nonce for user
curl http://localhost:8080/users/{userId}/nonce

# Response:
{
  "userId": "550e8400-e29b-41d4-a716-446655440000",
  "username": "@alice",
  "currentNonce": 5,
  "nextNonce": 6,
  "message": "Use nextNonce for your next transaction"
}
```

**Important:** Use `nextNonce` for your transaction, not `currentNonce`.

### 2. Create Transaction Message

Build a canonical message for signing:

```typescript
import { createTransactionMessage } from '@tana/cli/utils/crypto'

const transactionData = {
  type: 'transfer',
  from: '550e8400-e29b-41d4-a716-446655440000', // Your user ID
  to: '7c9e6679-7425-40de-944b-e07fc1f90ae7',   // Recipient ID
  amount: '100.00',
  currencyCode: 'USD',
  timestamp: Date.now(),
  nonce: 6 // The nextNonce from step 1
}

const message = createTransactionMessage(transactionData)
```

### 3. Sign the Message

Sign with your private key:

```typescript
import { signMessage } from '@tana/cli/utils/crypto'

const signature = await signMessage(
  message,
  userConfig.privateKey // From ~/.config/tana/users/@alice.json
)

// signature: "ed25519_a7f3b2c1d4e5f6a7b8c9d0e1f2a3b4c5..."
```

### 4. Submit Transaction

Send the signed transaction to the ledger:

```typescript
const response = await fetch('http://localhost:8080/transactions', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({
    type: 'transfer',
    from: '550e8400-e29b-41d4-a716-446655440000',
    to: '7c9e6679-7425-40de-944b-e07fc1f90ae7',
    amount: '100.00',
    currencyCode: 'USD',
    signature, // Include the signature
    timestamp: transactionData.timestamp,
    nonce: transactionData.nonce
  })
})
```

## Signature Verification

When the ledger receives a transaction, it:

1. **Validates timestamp** - Must be within 5 minutes
2. **Fetches user** - Gets user record by `from` field
3. **Checks nonce** - Must be exactly `currentNonce + 1`
4. **Verifies signature** - Recreates message and verifies with user's public key
5. **Checks balances** - Ensures sufficient funds (for transfers)

If any check fails, the transaction is rejected.

## Transaction Types

### User Creation

**Special case:** User creation doesn't require signature verification (no user exists yet to verify against). Instead:

- Nonce is always `0`
- Signature is verified against the `publicKey` field in the request itself
- This proves you control the private key for the public key you're registering

```typescript
const transactionData = {
  type: 'user_creation',
  from: '00000000-0000-0000-0000-000000000000', // System
  to: '00000000-0000-0000-0000-000000000000',   // Will be assigned
  timestamp: Date.now(),
  nonce: 0, // Always 0 for user creation
  contractInput: {
    username: '@alice',
    displayName: 'Alice',
    publicKey: 'ed25519_abc123...',
    bio: '',
    role: 'user'
  }
}
```

### Transfers

Transfer currency between accounts:

```typescript
{
  type: 'transfer',
  from: userId,
  to: recipientId,
  amount: '50.00',
  currencyCode: 'USD',
  signature: 'ed25519_...',
  timestamp: Date.now(),
  nonce: nextNonce
}
```

### Deposits

Request to deposit fiat currency into blockchain:

```typescript
{
  type: 'deposit',
  from: userId, // The requestor
  to: userId,   // Same (depositing to self)
  amount: '1000.00',
  currencyCode: 'CAD',
  signature: 'ed25519_...',
  timestamp: Date.now(),
  nonce: nextNonce,
  metadata: {
    // Optional: e-transfer details
  }
}
```

**Note:** Deposits create a pending transaction that must be confirmed by the sovereign user after receiving the actual fiat currency.

### Withdraws

Request to withdraw currency from blockchain to fiat:

```typescript
{
  type: 'withdraw',
  from: userId, // Your account (will be debited)
  to: userId,   // Same
  amount: '500.00',
  currencyCode: 'USD',
  signature: 'ed25519_...',
  timestamp: Date.now(),
  nonce: nextNonce,
  metadata: {
    email: 'alice@example.com' // Where to send e-transfer
  }
}
```

**Note:** Withdraws also require sovereign confirmation before the fiat currency is sent.

## Nonce Management

### What is a Nonce?

A **nonce** (number used once) is a counter that prevents replay attacks. Each user has their own nonce that:

- Starts at `0` when the user is created
- Increments by `1` every time a transaction is confirmed
- Must be sequential (can't skip numbers)

### Why Nonces Matter

Without nonces, an attacker could:
1. Capture a valid signed transaction
2. Re-submit it multiple times
3. Drain your account

With nonces, each transaction has a unique sequence number, so replays are detected and rejected.

### Handling Nonce Errors

If you get a nonce mismatch error:

```json
{
  "error": "Invalid nonce. Expected 7, got 6"
}
```

**Solutions:**

1. **Fetch fresh nonce** - Call `GET /users/:id/nonce` again
2. **Don't parallelize** - Wait for each transaction to be confirmed before submitting the next
3. **Check pending transactions** - Your transaction might be pending, increasing the nonce

### Pending Transactions

Nonces only increment when transactions are **confirmed** in a block. If you have pending transactions:

- Your `currentNonce` hasn't changed yet
- Use `nextNonce` based on the pending transaction count
- Wait for block production to confirm transactions

## CLI Signing Example

The `tana-cli` handles all signing automatically:

```bash
# Create user (signs automatically)
tana new user @alice
tana deploy user @alice

# Transfers (coming soon - will sign automatically)
tana transfer @alice @bob 50 USD
```

## TypeScript Library Example

```typescript
import * as ed from '@noble/ed25519'
import { createTransactionMessage, signMessage } from '@tana/cli/utils/crypto'

// 1. Load user config
const userConfig = readUserConfig('@alice')

// 2. Get current nonce
const nonceResponse = await fetch(`http://localhost:8080/users/${userConfig.id}/nonce`)
const { nextNonce } = await nonceResponse.json()

// 3. Create transaction data
const txData = {
  type: 'transfer',
  from: userConfig.id,
  to: recipientId,
  amount: '100.00',
  currencyCode: 'USD',
  timestamp: Date.now(),
  nonce: nextNonce
}

// 4. Sign the transaction
const message = createTransactionMessage(txData)
const signature = await signMessage(message, userConfig.privateKey)

// 5. Submit
await fetch('http://localhost:8080/transactions', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({ ...txData, signature })
})
```

## Security Best Practices

### Keep Private Keys Safe

- **Never share** your private key
- **Never commit** private keys to git
- **Store securely** in `~/.config/tana/users/` with proper permissions
- **Backup** to encrypted storage

### Verify Timestamps

Always use current timestamps:

```typescript
const timestamp = Date.now() // Current time in milliseconds
```

**Don't:**
- Use hardcoded timestamps
- Reuse old timestamps
- Set timestamps in the future

### Handle Nonces Carefully

- Always fetch fresh nonces before signing
- Wait for transactions to confirm before submitting more
- Don't parallelize transactions from the same account

### Verify Before Signing

Always review transaction details before signing:

```typescript
console.log('Transaction to sign:')
console.log(`  Type: ${txData.type}`)
console.log(`  To: ${txData.to}`)
console.log(`  Amount: ${txData.amount} ${txData.currencyCode}`)
console.log(`  Nonce: ${txData.nonce}`)

// Only sign if everything looks correct
const signature = await signMessage(message, privateKey)
```

## Error Reference

Common signature-related errors:

| Error | Cause | Solution |
|-------|-------|----------|
| `Invalid transaction signature` | Signature doesn't match | Regenerate signature with correct private key |
| `Invalid nonce. Expected X, got Y` | Wrong nonce | Fetch fresh nonce from `/users/:id/nonce` |
| `Transaction timestamp is too old` | Timestamp > 5 minutes old | Use current timestamp `Date.now()` |
| `Sender user not found` | Invalid `from` user ID | Verify user ID is correct |
| `Insufficient balance` | Not enough funds | Check balance before transferring |

## Next Steps

- **[Smart Contracts](/guides/smart-contracts/)** - Sign and deploy contracts
- **[API Reference](/tana-api/transactions/)** - Full transaction API docs
- **[CLI Commands](/tana-cli/intro/)** - Using the CLI for signed operations
