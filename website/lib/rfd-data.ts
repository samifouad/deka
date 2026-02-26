export type RFDStatus =
  | 'prediscussion'
  | 'ideation'
  | 'discussion'
  | 'published'
  | 'committed'
  | 'abandoned'

export interface RFD {
  number: number
  title: string
  status: RFDStatus
  authors: string[]
  created: string
  updated: string
  discussion: string // GitHub discussion URL
  content: string // Markdown content
  tags: string[]
}

export const rfds: RFD[] = [
  {
    number: 1,
    title: 'RFD Process and Template',
    status: 'committed',
    authors: ['Alex Rivera'],
    created: '2024-10-01',
    updated: '2024-10-15',
    discussion: 'https://github.com/conoda/tana/discussions/1',
    tags: ['process', 'meta'],
    content: `
# RFD 1: RFD Process and Template

## Metadata
- **Authors:** Alex Rivera
- **Status:** Committed
- **Created:** 2024-10-01
- **Updated:** 2024-10-15

## Overview

This RFD describes the Request for Discussion (RFD) process used by the tana project to propose, discuss, and document significant changes to the platform.

## Motivation

As tana grows, we need a structured way to:
- Propose and discuss architectural changes
- Document design decisions
- Enable public participation in governance
- Maintain a historical record of why decisions were made

## What is an RFD?

An RFD (Request for Discussion) is a document that:
- Proposes a significant change or new feature
- Provides context and motivation
- Explores alternatives and trade-offs
- Invites community feedback and discussion

## RFD Lifecycle

### Statuses

1. **Prediscussion** - Early idea, not yet ready for full discussion
2. **Ideation** - Exploring the problem space and potential solutions
3. **Discussion** - Open for community feedback and iteration
4. **Published** - Accepted as the direction forward
5. **Committed** - Fully implemented and deployed
6. **Abandoned** - No longer being pursued

### Process

1. **Create** - Author creates RFD with "prediscussion" status
2. **Refine** - Move to "ideation" when ready for feedback
3. **Discuss** - Open GitHub discussion, status becomes "discussion"
4. **Decide** - After consensus, mark as "published"
5. **Implement** - Build it, mark as "committed" when done

## RFD Template

Each RFD should include:

- **Metadata** (number, authors, status, dates)
- **Overview** - Brief summary
- **Motivation** - Why is this needed?
- **Proposal** - What exactly are we proposing?
- **Alternatives** - What else was considered?
- **Implementation** - How will we build it?
- **Open Questions** - What still needs discussion?

## Who Can Submit RFDs?

Anyone! RFDs are open to:
- Core team members
- Contributors
- Community members
- Users

## Discussion

All RFD discussions happen on GitHub. Each RFD links to its discussion thread where community members can ask questions, provide feedback, and help shape the proposal.

## Inspiration

This process is inspired by [Oxide Computer Company's RFD process](https://rfd.shared.oxide.computer/), which has proven effective for transparent, community-driven decision making.
    `.trim()
  },
  {
    number: 2,
    title: 'Multi-Validator Consensus Mechanism',
    status: 'discussion',
    authors: ['Alex Rivera', 'Jordan Kim'],
    created: '2024-11-01',
    updated: '2024-11-14',
    discussion: 'https://github.com/conoda/tana/discussions/2',
    tags: ['consensus', 'blockchain', 'architecture'],
    content: `
# RFD 2: Multi-Validator Consensus Mechanism

## Metadata
- **Authors:** Alex Rivera, Jordan Kim
- **Status:** Discussion
- **Created:** 2024-11-01
- **Updated:** 2024-11-14

## Overview

This RFD proposes a consensus mechanism to enable multiple validators to participate in block production while maintaining deterministic transaction ordering and preventing chain forks.

## Motivation

Currently, tana only supports single-validator deployments. To achieve true decentralization and fault tolerance, we need:

- Multiple validators producing blocks
- Consensus on transaction ordering
- Fork resolution mechanisms
- Byzantine fault tolerance

## Current State

**What Works:**
- Single validator produces blocks reliably
- Transaction queue (Redis Streams) supports multiple consumers
- Deterministic ordering via Stream IDs

**What Doesn't:**
- No voting mechanism for block validity
- No fork resolution
- No chain synchronization between validators

## Proposal

### Architecture

We propose a **Practical Byzantine Fault Tolerance (PBFT)** inspired consensus with these components:

1. **Leader Election**
   - Round-robin leader selection based on validator set
   - Leader proposes blocks
   - Other validators vote on proposed blocks

2. **Block Voting**
   - 2/3+ validators must approve each block
   - Votes signed with Ed25519 keys
   - Timeout triggers leader rotation

3. **Fork Resolution**
   - Longest chain rule with vote weight
   - Validators sync from peers on startup
   - Orphaned blocks stored for analysis

### Implementation Phases

**Phase 1: Leader Election (Week 1)**
- Implement deterministic leader selection
- Add validator registration system
- Build leader rotation logic

**Phase 2: Block Voting (Week 2)**
- Add voting message types
- Implement vote aggregation
- Build 2/3+ threshold logic

**Phase 3: Synchronization (Week 3)**
- Peer discovery mechanism
- Block sync protocol
- State verification

**Phase 4: Fork Resolution (Week 4)**
- Chain weight calculation
- Orphan block handling
- Reorganization logic

## Alternatives Considered

### 1. Proof of Stake

**Pros:**
- Energy efficient
- Economic security

**Cons:**
- Complex validator economics
- "Nothing at stake" problem
- Not aligned with multi-currency model

### 2. Proof of Work

**Pros:**
- Well understood
- Battle tested

**Cons:**
- Energy intensive
- Centralization via mining pools
- Not suitable for permissioned networks

### 3. Tendermint Consensus

**Pros:**
- Proven technology
- Good tooling

**Cons:**
- Heavy dependency
- Opinionated architecture
- Less control over customization

## Implementation Details

### Validator Registration

\`\`\`typescript
interface Validator {
  id: string
  publicKey: string
  endpoint: string
  stake: number // For future PoS migration
  joinedAt: number
}
\`\`\`

### Block Proposal Format

\`\`\`typescript
interface BlockProposal {
  height: number
  leader: string
  transactions: Transaction[]
  previousHash: string
  timestamp: number
  signature: string
}
\`\`\`

### Vote Message

\`\`\`typescript
interface BlockVote {
  blockHash: string
  height: number
  validator: string
  approved: boolean
  signature: string
}
\`\`\`

## Security Considerations

- **Byzantine Validators:** System tolerates up to 1/3 malicious validators
- **Sybil Attacks:** Validator registration requires stake/KYC
- **Network Partitions:** Timeout and reorganization logic handles splits
- **Long-Range Attacks:** Checkpointing every 1000 blocks

## Performance Impact

- **Block Time:** Increases from ~6s to ~10s due to voting
- **Throughput:** Should maintain 100,000+ tx/sec
- **Network Overhead:** ~1KB per vote, ~10 votes per block

## Testing Strategy

1. Unit tests for leader election
2. Integration tests with 3, 5, 10 validators
3. Chaos testing (network partitions, validator failures)
4. Testnet deployment before mainnet

## Open Questions

1. Should validator set be dynamic or fixed?
2. What's the minimum/maximum number of validators?
3. How do we handle validator updates without downtime?
4. Should we support light clients?

## Timeline

- **Week 1-4:** Implementation
- **Week 5-6:** Testing and bug fixes
- **Week 7:** Testnet deployment
- **Week 8:** Mainnet migration

## References

- [PBFT Paper](http://pmg.csail.mit.edu/papers/osdi99.pdf)
- [Tendermint Docs](https://docs.tendermint.com/)
- [Ethereum Consensus](https://ethereum.org/en/developers/docs/consensus-mechanisms/)
    `.trim()
  },
  {
    number: 3,
    title: 'Mobile App Offline Mode',
    status: 'ideation',
    authors: ['Sarah Chen'],
    created: '2024-11-12',
    updated: '2024-11-15',
    discussion: 'https://github.com/conoda/tana/discussions/3',
    tags: ['mobile', 'offline', 'ux'],
    content: `
# RFD 3: Mobile App Offline Mode

## Metadata
- **Authors:** Sarah Chen
- **Status:** Ideation
- **Created:** 2024-11-12
- **Updated:** 2024-11-15

## Overview

This RFD proposes adding offline capabilities to the tana mobile app, allowing users to queue transactions and view recent activity without an internet connection.

## Motivation

Users often encounter situations with limited or no connectivity:
- Traveling on airplanes
- Rural areas with poor coverage
- International roaming restrictions
- Underground locations (subways, basements)

Currently, the tana app is completely unusable without internet. This creates a poor user experience and limits adoption.

## Proposal

### Offline Capabilities

**What Should Work Offline:**
1. View transaction history (cached locally)
2. View account balances (last known state)
3. Queue outgoing transactions for when online
4. View contact list
5. Generate payment QR codes

**What Requires Online:**
1. Confirming transactions
2. Receiving payments
3. Refreshing balances
4. Authentication/login

### Implementation

**Local Database (SQLite)**
- Cache last 100 transactions
- Store current balance snapshot
- Queue pending transactions
- Store contact information

**Sync Strategy**
- Background sync when connection restored
- Pull recent transactions
- Submit queued transactions
- Update balance

**UI Indicators**
- Clear "offline mode" banner
- Pending transaction badges
- Last sync timestamp
- Data staleness warnings

## Alternatives

### 1. Read-Only Offline Mode

Only allow viewing data, no transaction creation.

**Pros:** Simpler, safer
**Cons:** Limited utility

### 2. Full Blockchain Sync

Download entire blockchain to device.

**Pros:** Complete offline functionality
**Cons:** Huge storage requirements, impractical for mobile

### 3. No Offline Mode

Keep current behavior.

**Pros:** Simplest to maintain
**Cons:** Poor UX, limits adoption

## Open Questions

1. How long should we cache transaction history?
2. Should we allow offline payment requests?
3. How do we handle conflicts (e.g., balance spent elsewhere)?
4. What's the maximum queue size for pending transactions?
5. Should we support peer-to-peer transaction broadcasting?

## Security Considerations

- Encrypted local storage using device keychain
- Biometric authentication before offline access
- Rate limiting on queued transactions
- Tamper detection for cached data

## Implementation Estimate

- **2 weeks** for local database and caching
- **1 week** for sync logic
- **1 week** for UI updates
- **1 week** for testing

## Next Steps

Looking for feedback on:
- Which offline features are most important?
- Should we use SQLite or another local database?
- How to handle edge cases and conflicts?

Please comment on the [discussion thread](https://github.com/conoda/tana/discussions/3)!
    `.trim()
  }
]

export function getRFDs(): RFD[] {
  return rfds.sort((a, b) => b.number - a.number)
}

export function getRFD(number: number): RFD | undefined {
  return rfds.find(rfd => rfd.number === number)
}

export function getRFDsByStatus(status: RFDStatus): RFD[] {
  return rfds.filter(rfd => rfd.status === status)
}

export function getRFDsByTag(tag: string): RFD[] {
  return rfds.filter(rfd => rfd.tags.includes(tag))
}

export function getStatusColor(status: RFDStatus): string {
  switch (status) {
    case 'prediscussion':
      return 'bg-gray-500/10 text-gray-500'
    case 'ideation':
      return 'bg-blue-500/10 text-blue-500'
    case 'discussion':
      return 'bg-yellow-500/10 text-yellow-500'
    case 'published':
      return 'bg-purple-500/10 text-purple-500'
    case 'committed':
      return 'bg-green-500/10 text-green-500'
    case 'abandoned':
      return 'bg-red-500/10 text-red-500'
  }
}
