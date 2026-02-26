export interface BlogPost {
  slug: string
  title: string
  excerpt: string
  content: string
  date: string
  author: {
    name: string
    role: string
  }
  tags: string[]
  readTime: string
}

export const blogPosts: BlogPost[] = [
  {
    slug: 'introducing-tana',
    title: 'Introducing tana: The Future of Money and Identity',
    excerpt: 'Today we\'re excited to announce tana, a revolutionary platform that reimagines how people send money and manage their digital identity. No passwords, no fees, no compromises.',
    content: `
# Introducing tana: The Future of Money and Identity

Today we're excited to announce **tana**, a revolutionary platform that reimagines how people send money and manage their digital identity.

## The Problem

Traditional banking and payment apps are stuck in the past:
- High transaction fees eat into every payment
- Password-based authentication is both insecure and frustrating
- Centralized systems mean you don't own your data
- International transfers are slow and expensive

## The Solution

tana fixes all of this with three core innovations:

### 1. Zero Transaction Fees

We only charge a one-time withdrawal fee. Send money as many times as you want, to anyone in the world, for free. Keep 100% of your money.

### 2. Passwordless Authentication

Your phone is your identity. Using cryptographic signatures and QR codes, you can securely log in from any device without ever remembering a password.

### 3. Decentralized Architecture

Built on blockchain technology, tana gives you complete control over your data. We can't access it, we can't sell it, we can't lose it.

## What's Next

This is just the beginning. In the coming months, we'll be rolling out merchant tools, developer APIs, and enterprise features. Stay tuned.

**Download tana today and join the revolution.**
    `.trim(),
    date: '2024-11-15',
    author: {
      name: 'Alex Rivera',
      role: 'CEO & Co-founder'
    },
    tags: ['announcement', 'product'],
    readTime: '3 min'
  },
  {
    slug: 'why-we-built-tana',
    title: 'Why We Built tana: A Personal Story',
    excerpt: 'The story behind tana started with a frustrating experience trying to send money to my family overseas. Here\'s why we decided to build something better.',
    content: `
# Why We Built tana: A Personal Story

The story behind tana started with a frustrating experience trying to send money to my family overseas.

## The Spark

Three years ago, I tried to send $500 to my parents in another country. The process was:
- 30 minutes of filling out forms
- $25 in fees
- 3-5 business days for the money to arrive
- My parents had to drive to a physical location to pick it up

It felt absurd. We can send messages instantly for free. Why can't we do the same with money?

## The Research

I started digging into the payment industry and discovered some shocking facts:

- The average person pays **$200-300 per year** in payment fees
- Traditional banks spend **$400-600 per customer per year** on password resets and account recovery
- **81%** of people reuse passwords across multiple sites because they can't remember them all
- International remittance fees total **$50 billion per year** globally

## The Vision

We realized the problem wasn't just about money—it was about identity and control. People needed:

1. A way to send money without fees
2. A secure identity system that doesn't rely on passwords
3. Ownership of their own data

So we built tana.

## What We've Learned

Building tana has been challenging but rewarding. We've learned that people are ready for a new approach to money and identity. They're tired of fees, passwords, and giving up control.

The future is decentralized, passwordless, and fee-free. We're just getting started.
    `.trim(),
    date: '2024-11-10',
    author: {
      name: 'Alex Rivera',
      role: 'CEO & Co-founder'
    },
    tags: ['story', 'vision'],
    readTime: '5 min'
  },
  {
    slug: 'for-merchants-zero-fees',
    title: 'For Merchants: How Zero Fees Changes Everything',
    excerpt: 'We built tana for consumers first, but merchants are loving it too. Here\'s how eliminating transaction fees can transform your business.',
    content: `
# For Merchants: How Zero Fees Changes Everything

We built tana for consumers first, but merchants are loving it too. Here's how eliminating transaction fees can transform your business.

## The Hidden Cost of Payments

Most merchants don't realize how much they're paying in fees:

- **Credit cards:** 2.9% + 30¢ per transaction
- **PayPal:** 2.9% + 30¢ per transaction
- **Square:** 2.6% + 10¢ per transaction

For a business doing $100,000 in annual revenue, that's **$2,900 in fees**. Every single year.

## The tana Difference

With tana, you pay:
- **$0** monthly fees
- **0%** transaction fees
- **One-time withdrawal fee** only when you cash out

That's it. Keep 100% of every sale.

## Real Impact

Let's look at a real example. A coffee shop doing $500,000 in annual revenue:

**With Shopify + Stripe:**
- Monthly fees: $29/month = $348/year
- Transaction fees: $500,000 × 2.9% = $14,500/year
- **Total: $14,848/year**

**With tana:**
- Monthly fees: $0
- Transaction fees: $0
- Withdrawal fees: ~$50/year
- **Total: $50/year**

**Savings: $14,798/year**

## Beyond Fees

But it's not just about money. tana also includes:

- Built-in team chat (no Slack subscription needed)
- Advanced analytics (normally $299/month on Shopify)
- Customer service tools (no Zendesk needed)
- Unlimited team members (no per-user fees)

## Join the Movement

Thousands of merchants have already made the switch. Ready to keep 100% of your revenue?

[Start your free store →](#)
    `.trim(),
    date: '2024-11-05',
    author: {
      name: 'Jordan Kim',
      role: 'Head of Merchant Success'
    },
    tags: ['merchants', 'business'],
    readTime: '4 min'
  }
]

export function getBlogPosts(): BlogPost[] {
  return blogPosts.sort((a, b) => new Date(b.date).getTime() - new Date(a.date).getTime())
}

export function getBlogPost(slug: string): BlogPost | undefined {
  return blogPosts.find(post => post.slug === slug)
}

export function getBlogPostsByTag(tag: string): BlogPost[] {
  return blogPosts.filter(post => post.tags.includes(tag))
}
