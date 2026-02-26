---
title: How Blockchain Works
description: A simple introduction to blockchain concepts
sidebar:
  order: 2
---

If you're new to blockchain, this guide explains the core concepts in plain language.

## What is a Blockchain?

A blockchain is a **shared database** that multiple computers maintain together. Instead of one company controlling all the data, the network participants collectively agree on what's true.

Think of it like a Google Doc where:
- Everyone can see the full history of changes
- No one can secretly edit past entries
- The group must agree before adding new content

## Why Use a Blockchain?

Traditional databases have a single owner who can:
- Change records without notice
- Deny access to certain users
- Shut down the system entirely

A blockchain provides:
- **Transparency** - Everyone sees the same data
- **Immutability** - Past records can't be changed
- **Availability** - No single point of failure
- **Trust** - Rules are enforced by code, not people

## Core Concepts

### Transactions

A **transaction** is a request to change something:
- Transfer money between accounts
- Update a piece of data
- Deploy a new program

Every transaction includes:
- Who is making the request (sender)
- What they want to do (action)
- A digital signature (proof it's really them)

### Blocks

Transactions are grouped into **blocks**. Each block contains:
- A batch of transactions
- A timestamp
- A reference to the previous block
- A unique fingerprint (hash)

This chain of references is why it's called a "block-chain":

```
[Block 0] ← [Block 1] ← [Block 2] ← [Block 3]
 Genesis      10 txs      25 txs      15 txs
```

### Consensus

How do multiple computers agree on the next block? They use **consensus** - a process where validators:

1. Receive transactions from users
2. Propose a new block
3. Vote on whether it's valid
4. Add it to their copy of the chain

If validators disagree, the majority wins. This prevents any single validator from cheating.

### Digital Signatures

How do you prove a transaction is really from you? **Digital signatures** using cryptography:

1. You have a **private key** (secret, like a password)
2. You have a **public key** (shared, like a username)
3. You sign transactions with your private key
4. Anyone can verify with your public key

This is more secure than passwords because:
- The private key never leaves your device
- Each signature is unique to the transaction
- Forgery is mathematically impossible

## Tana's Approach

### Validators

In Tana, validators are servers that:
- Run the consensus protocol
- Store the blockchain data
- Execute smart contracts

Anyone can run a validator if the network sovereign approves them.

### Smart Contracts

A **smart contract** is a program stored on the blockchain. When triggered, it executes automatically and predictably.

In Tana, contracts are written in TypeScript:

```typescript
// A simple voting contract
export async function contract() {
  const caller = context.caller()
  const hasVoted = await kv.get(`voted:${caller.id}`)

  if (hasVoted) {
    return { error: 'Already voted' }
  }

  await kv.put(`voted:${caller.id}`, 'true')
  return { success: true }
}
```

### State

The **state** is the current data stored on the blockchain:
- User accounts and balances
- Contract storage (key-value pairs)
- System configuration

State changes only happen through valid transactions that are included in blocks.

## Common Questions

### Is blockchain slow?

Traditional blockchains can be slow because every transaction goes through consensus. Tana uses a hybrid model:
- **Critical operations** → Blockchain (secure but slower)
- **Read operations** → HTTP servers (fast)

### Does blockchain use a lot of energy?

Some blockchains (like Bitcoin) use "proof of work" which requires massive computation. Tana uses "proof of authority" - validators are approved servers, not competing miners. This uses minimal energy.

### Is everything public?

In Tana, transaction data is visible to validators. For privacy-sensitive applications, you can run a private network where only authorized parties are validators.

### Do I need cryptocurrency?

No. Tana doesn't require a native token. You can use it without any cryptocurrency, or create your own currencies as needed.

## Next Steps

- [What is Tana?](/docs/guides/what-is-tana/) - Tana-specific concepts
- [Quick Start](/docs/guides/quickstart/) - Try it yourself
