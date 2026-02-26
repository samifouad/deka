---
title: System Flows
description: How data flows through Tana's services
---

This guide maps out the key user actions and how they flow through Tana's services.

## Core User Actions

### 1. User Creates Account

```
User submits:
  - username
  - display_name
  - public_key
    ↓
POST /users
    ↓
Ledger Service validates:
  - username unique?
  - valid format?
    ↓
PostgreSQL INSERT
    ↓
Returns: user object with ID
```

**Services Required:**
- ✅ Ledger API
- ✅ PostgreSQL

**Status:** ✅ **READY**

### 2. User Checks Balance

```
User queries: GET /balances?ownerId=alice&currencyCode=USD
    ↓
Ledger Service queries DB
    ↓
PostgreSQL SELECT
    ↓
Returns: { amount: "1000.00", currencyCode: "USD" }
```

**Services Required:**
- ✅ Ledger API
- ✅ PostgreSQL

**Status:** ✅ **READY**

### 3. User Sends Funds

```
User submits: POST /transactions
  {
    from: "alice",
    to: "bob",
    amount: "100.00",
    currency: "USD"
  }
    ↓
Ledger Service validates:
  - Alice has balance >= 100?
  - Bob exists?
    ↓
BEGIN TRANSACTION
  UPDATE balances SET amount = amount - 100 WHERE owner_id = 'alice'
  UPDATE balances SET amount = amount + 100 WHERE owner_id = 'bob'
  INSERT INTO transactions (...)
COMMIT
    ↓
Returns: transaction ID
```

**Services Required:**
- ✅ Ledger API
- ✅ PostgreSQL

**Status:** ✅ **READY**

### 4. User Tests Contract in Playground

```
User writes TypeScript in browser
    ↓
Clicks "Run"
    ↓
Browser sandbox executes:
  - Transpiles TS → JS
  - Runs in isolated context
  - tana/* modules available
    ↓
Contract reads data:
  GET /balances (read-only)
    ↓
Contract simulates writes:
  tx.transfer() - staged locally
  tx.execute() - returns mock result
    ↓
Browser displays output
```

**Services Required:**
- ✅ Playground Website
- ✅ Ledger API (read-only)
- ✅ Browser Sandbox

**Status:** ✅ **READY**

### 5. User Deploys Contract (via CLI)

```
User runs: tana deploy contract ./my-contract.ts
    ↓
CLI validates contract code
    ↓
POST /contracts
  {
    code: "...",
    owner: "alice"
  }
    ↓
Ledger Service:
  - Stores contract code
  - Generates contract ID
    ↓
PostgreSQL INSERT
    ↓
Returns: contract_id
```

**Services Required:**
- ✅ CLI
- ✅ Ledger API
- ✅ PostgreSQL

**Status:** ⏳ **IN PROGRESS**

### 6. User Executes Contract (Future)

```
User calls: POST /contracts/:id/execute { args: {...} }
    ↓
Contracts Service:
  - Loads contract code from DB
  - Spawns Rust runtime process
    ↓
Runtime executes:
  - Reads state via tana/block
  - Stages changes via tana/tx
  - Returns execution result
    ↓
Contracts Service validates:
  - Gas limit not exceeded?
  - Valid state transitions?
    ↓
If valid:
  POST /transactions (apply changes)
    ↓
Returns: execution result
```

**Services Required:**
- ⏳ Contracts Service
- ✅ Rust Runtime
- ✅ Ledger API

**Status:** ⏳ **PLANNED**

## Data Flow Patterns

### Pattern 1: Read-Only Query

```
User → API → PostgreSQL → Response
```

**Example:** Get balance, list users, query transactions

**Status:** ✅ Working

### Pattern 2: Direct State Change

```
User → API → Validate → PostgreSQL Transaction → Response
```

**Example:** Create user, transfer funds

**Status:** ✅ Working

### Pattern 3: Contract Execution (Future)

```
User → API → Runtime → Validate → PostgreSQL → Response
         ↓        ↓
    Load Code  Execute
```

**Example:** Deploy contract, execute contract

**Status:** ⏳ Planned

## Service Dependencies

### Current Architecture

```
CLI/Playground
    ↓ HTTP
Ledger Service
    ↓ TCP
PostgreSQL
```

**Status:** ✅ Fully operational

### Future Architecture

```
CLI/Playground
    ↓ HTTP
Ledger Service
    ↓ Subprocess
Runtime (Rust)
    ↓ HTTP
Ledger Service
    ↓ TCP
PostgreSQL + Redis
```

**Status:** ⏳ Partially implemented

## Transaction Lifecycle

### 1. Submission

```
User creates transaction
    ↓
Validate locally (signature, format)
    ↓
Submit to ledger API
```

### 2. Validation

```
Ledger receives transaction
    ↓
Validate:
  - Valid signature?
  - Sufficient balance?
  - Valid recipient?
    ↓
If invalid: Reject
If valid: Continue
```

### 3. Execution

```
Begin database transaction
    ↓
Update balances
    ↓
Insert transaction record
    ↓
Commit
```

### 4. Confirmation

```
Transaction included in block
    ↓
State root updated
    ↓
Transaction confirmed
```

## Next Steps

- [Data Model](/guides/core/data-model/) - Understand the data structures
- [Blocks & Blockchain](/guides/core/blocks/) - Learn about block production
- [CLI Reference](/tana-cli/intro/) - Use the command-line interface
