'use client'

import { useState } from 'react'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Copy, Check } from 'lucide-react'

interface CodeExample {
  id: string
  title: string
  description: string
  code: string
  result: any
  category: string
}

const codeExamples: CodeExample[] = [
  {
    id: 'counter',
    title: "Counter Contract",
    description: "Simple counter with persistent storage",
    category: "Basic",
    code: `import { console } from 'tana:core'
import { data } from 'tana:data'

const current = await data.get('counter')
const count = current ? parseInt(current) : 0

await data.set('counter', String(count + 1))
await data.commit()

console.log('Counter:', count + 1)`,
    result: {
      output: "Counter: 42",
      storage: { counter: "42" },
      transaction: { type: "contract_execution", status: "success", gas_used: "1,234" }
    }
  },
  {
    id: 'transfer',
    title: "Currency Transfer",
    description: "Transfer USD between users",
    category: "Finance",
    code: `import { tx } from 'tana:tx'
import { context } from 'tana:context'

await tx.transfer({
  from: context.caller,
  to: "usr_bob",
  amount: "100",
  currencyCode: "USD"
})

console.log('Transfer completed!')`,
    result: {
      output: "Transfer completed!",
      balances: { "usr_alice": { USD: "900" }, "usr_bob": { USD: "200" } },
      transaction: { type: "transfer", from: "usr_alice", to: "usr_bob", amount: "100", currency: "USD", status: "confirmed" }
    }
  },
  {
    id: 'user-registry',
    title: "User Registry",
    description: "Register new users on blockchain",
    category: "Identity",
    code: `import { data } from 'tana:data'
import { context } from 'tana:context'

const userData = {
  id: context.caller,
  name: context.input.name,
  email: context.input.email,
  created: Date.now()
}

await data.set(\`user:\${context.caller}\`, JSON.stringify(userData))
await data.commit()

console.log('User registered:', userData.name)`,
    result: {
      output: "User registered: Alice Johnson",
      storage: {
        "user:usr_alice": {
          id: "usr_alice",
          name: "Alice Johnson",
          email: "alice@example.com",
          created: 1703123456789
        }
      },
      transaction: { type: "user_registration", user_id: "usr_alice", status: "success" }
    }
  },
  {
    id: 'voting',
    title: "Voting System",
    description: "Decentralized voting mechanism",
    category: "Governance",
    code: `import { data } from 'tana:data'
import { context } from 'tana:context'

const proposal = context.input.proposalId
const vote = context.input.vote // 'yes' or 'no'

const key = \`vote:\${proposal}:\${context.caller}\`
await data.set(key, vote)
await data.commit()

console.log('Vote recorded for proposal:', proposal)`,
    result: {
      output: "Vote recorded for proposal: prop_001",
      storage: { "vote:prop_001:usr_alice": "yes" },
      transaction: { type: "vote", proposal: "prop_001", voter: "usr_alice", vote: "yes" }
    }
  },
  {
    id: 'nft',
    title: "NFT Minting",
    description: "Create and manage NFTs",
    category: "Digital Assets",
    code: `import { data } from 'tana:data'
import { context } from 'tana:context'

const nftId = context.input.nftId
const metadata = context.input.metadata

const nft = {
  id: nftId,
  owner: context.caller,
  metadata: metadata,
  created: Date.now()
}

await data.set(\`nft:\${nftId}\`, JSON.stringify(nft))
await data.commit()

console.log('NFT minted:', nftId)`,
    result: {
      output: "NFT minted: nft_001",
      storage: {
        "nft:nft_001": {
          id: "nft_001",
          owner: "usr_alice",
          metadata: { name: "My First NFT", description: "A beautiful digital artwork" },
          created: 1703123456789
        }
      },
      transaction: { type: "nft_mint", nft_id: "nft_001", owner: "usr_alice" }
    }
  },
  {
    id: 'escrow',
    title: "Escrow Service",
    description: "Secure escrow for transactions",
    category: "Finance",
    code: `import { data } from 'tana:data'
import { tx } from 'tana:tx'
import { context } from 'tana:context'

const escrow = {
  id: context.input.escrowId,
  buyer: context.input.buyer,
  seller: context.input.seller,
  amount: context.input.amount,
  currency: context.input.currency,
  status: 'pending'
}

await data.set(\`escrow:\${escrow.id}\`, JSON.stringify(escrow))
await tx.transfer({
  from: context.input.buyer,
  to: "escrow_account",
  amount: escrow.amount,
  currencyCode: escrow.currency
})

await data.commit()`,
    result: {
      output: "Escrow created and funded",
      storage: {
        "escrow:esc_001": {
          id: "esc_001",
          buyer: "usr_alice",
          seller: "usr_bob",
          amount: "500",
          currency: "USD",
          status: "pending"
        }
      },
      transaction: { type: "escrow_create", escrow_id: "esc_001", amount: "500 USD" }
    }
  },
  {
    id: 'multisig',
    title: "Multi-Sig Wallet",
    description: "Multi-signature wallet security",
    category: "Security",
    code: `import { data } from 'tana:data'
import { context } from 'tana:context'

const walletId = context.input.walletId
const requiredSigs = context.input.requiredSignatures
const owners = context.input.owners

const wallet = {
  id: walletId,
  owners: owners,
  requiredSignatures: requiredSigs,
  pendingTransactions: [],
  nonce: 0
}

await data.set(\`wallet:\${walletId}\`, JSON.stringify(wallet))
await data.commit()

console.log('Multi-sig wallet created:', walletId)`,
    result: {
      output: "Multi-sig wallet created: wallet_001",
      storage: {
        "wallet:wallet_001": {
          id: "wallet_001",
          owners: ["usr_alice", "usr_bob", "usr_charlie"],
          requiredSignatures: 2,
          pendingTransactions: [],
          nonce: 0
        }
      },
      transaction: { type: "multisig_create", wallet_id: "wallet_001" }
    }
  },
  {
    id: 'token',
    title: "Token Contract",
    description: "Create custom tokens",
    category: "DeFi",
    code: `import { data } from 'tana:data'
import { context } from 'tana:context'

const tokenName = context.input.name
const symbol = context.input.symbol
const totalSupply = context.input.totalSupply

const token = {
  name: tokenName,
  symbol: symbol,
  totalSupply: totalSupply,
  balances: { [context.caller]: totalSupply }
}

await data.set(\`token:\${symbol}\`, JSON.stringify(token))
await data.commit()

console.log('Token created:', symbol)`,
    result: {
      output: "Token created: TKN",
      storage: {
        "token:TKN": {
          name: "My Token",
          symbol: "TKN",
          totalSupply: "1000000",
          balances: { "usr_alice": "1000000" }
        }
      },
      transaction: { type: "token_create", symbol: "TKN", supply: "1000000" }
    }
  },
  {
    id: 'marketplace',
    title: "Marketplace",
    description: "Decentralized marketplace",
    category: "Commerce",
    code: `import { data } from 'tana:data'
import { context } from 'tana:context'

const listing = {
  id: context.input.listingId,
  seller: context.caller,
  item: context.input.item,
  price: context.input.price,
  currency: context.input.currency,
  status: 'active'
}

await data.set(\`listing:\${listing.id}\`, JSON.stringify(listing))
await data.commit()

console.log('Item listed:', listing.item.name)`,
    result: {
      output: "Item listed: Digital Art #001",
      storage: {
        "listing:list_001": {
          id: "list_001",
          seller: "usr_alice",
          item: { name: "Digital Art #001", description: "Beautiful digital artwork" },
          price: "100",
          currency: "USD",
          status: "active"
        }
      },
      transaction: { type: "listing_create", listing_id: "list_001", price: "100 USD" }
    }
  },
  {
    id: 'dao',
    title: "DAO Framework",
    description: "Decentralized autonomous organization",
    category: "Governance",
    code: `import { data } from 'tana:data'
import { context } from 'tana:context'

const dao = {
  name: context.input.name,
  members: [context.caller],
  proposals: [],
  treasury: "0",
  votingPeriod: context.input.votingPeriod || 7 * 24 * 60 * 60 * 1000 // 7 days
}

await data.set(\`dao:\${context.input.name}\`, JSON.stringify(dao))
await data.commit()

console.log('DAO created:', dao.name)`,
    result: {
      output: "DAO created: MyDAO",
      storage: {
        "dao:MyDAO": {
          name: "MyDAO",
          members: ["usr_alice"],
          proposals: [],
          treasury: "0",
          votingPeriod: 604800000
        }
      },
      transaction: { type: "dao_create", dao_name: "MyDAO" }
    }
  }
]

interface CodeExampleCarouselProps {
  onExampleSelect: (example: CodeExample) => void
  selectedExample: CodeExample | null
}

export function CodeExampleCarousel({ onExampleSelect, selectedExample }: CodeExampleCarouselProps) {
  const [copiedId, setCopiedId] = useState<string | null>(null)

  const copyToClipboard = async (code: string, id: string) => {
    try {
      await navigator.clipboard.writeText(code)
      setCopiedId(id)
      setTimeout(() => setCopiedId(null), 2000)
    } catch (err) {
      console.error('Failed to copy code:', err)
    }
  }

  return (
    <div className="relative">
      <div className="absolute left-0 top-0 bottom-0 w-8 bg-gradient-to-r from-slate-950 to-transparent z-10 pointer-events-none"></div>
      <div className="absolute right-0 top-0 bottom-0 w-8 bg-gradient-to-l from-slate-950 to-transparent z-10 pointer-events-none"></div>
      
      {/* Endlessly scrollable horizontal list */}
      <div className="flex gap-4 overflow-x-auto pb-4 scrollbar-hide animate-scroll">
        {[...codeExamples, ...codeExamples].map((example, index) => (
          <Card
            key={`${example.id}-${index}`}
            className={`min-w-80 bg-slate-900/50 border-blue-400/20 backdrop-blur-sm hover:border-blue-400/40 transition-all duration-300 cursor-pointer flex-shrink-0 ${
              selectedExample?.id === example.id ? 'border-blue-400/60 bg-slate-900/70' : ''
            }`}
            onClick={() => onExampleSelect(example)}
          >
            <CardHeader className="pb-3">
              <div className="flex items-start justify-between">
                <div className="flex-1">
                  <CardTitle className="text-blue-200 text-lg">{example.title}</CardTitle>
                  <CardDescription className="text-blue-300/70 text-sm mt-1">
                    {example.description}
                  </CardDescription>
                </div>
                <span className="text-xs bg-blue-600/20 text-blue-300 px-2 py-1 rounded border border-blue-400/30">
                  {example.category}
                </span>
              </div>
            </CardHeader>
            <CardContent className="pt-0">
              <div className="relative">
                <pre className="bg-slate-950/50 border border-blue-400/10 rounded-lg p-3 overflow-x-auto text-xs max-h-32">
                  <code className="text-blue-200">
                    {example.code.split('\n').slice(0, 6).join('\n')}
                    {example.code.split('\n').length > 6 && '\n...'}
                  </code>
                </pre>
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={(e) => {
                    e.stopPropagation()
                    copyToClipboard(example.code, example.id)
                  }}
                  className="absolute top-2 right-2 text-blue-300 hover:text-blue-200 hover:bg-blue-900/20 p-1 h-6 w-6"
                >
                  {copiedId === example.id ? <Check className="w-3 h-3" /> : <Copy className="w-3 h-3" />}
                </Button>
              </div>
            </CardContent>
          </Card>
        ))}
      </div>
    </div>
  )
}

export { codeExamples }