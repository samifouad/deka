---
title: Development Setup
description: Build Tana from source for development
sidebar:
  order: 2
---

This guide is for **contributors** who want to build Tana from source. If you just want to use Tana, see the [Quick Start](/guides/quickstart/) instead.

## Prerequisites

- [Bun](https://bun.sh) >= 1.0 (TypeScript runtime)
- [Rust](https://rustup.rs) >= 1.70 (for runtime binary)
- [Docker](https://docker.com) (for PostgreSQL and Redis)
- [mprocs](https://github.com/pvolok/mprocs) (optional, for multi-process management)

## Clone Repository

The Tana project is organized as multiple repositories under the `tananetwork` organization:

```bash
# Clone the main development environment
git clone https://github.com/tananetwork/tana-dev-env.git
cd tana-dev-env

# Or clone individual repos as needed:
# git clone https://github.com/tananetwork/tana-cli.git
# git clone https://github.com/tananetwork/tana-runtime.git
# git clone https://github.com/tananetwork/tana-edge.git
```

## Install Dependencies

```bash
# Install all workspace dependencies
bun install
```

## Build Rust Runtime

```bash
cd runtime
cargo build --release
cd ..
```

This creates the `tana-runtime` binary at `runtime/target/release/tana-runtime`.

## Build CLI (Optional)

```bash
cd cli
bun run make
cd ..
```

This creates a standalone `tana` binary at `cli/dist/tana`.

## Start Development Environment

### Option 1: Quick Start with Tana CLI

```bash
# Create a user
bun cli/main.ts new user @alice --name "Alice"

# Start Tana
bun cli/main.ts start
```

This automatically starts all required services:
- ✅ PostgreSQL
- ✅ Redis
- ✅ Mesh Service
- ✅ T4 Service
- ✅ Ledger Service - http://localhost:8080

### Option 2: All Services with mprocs

```bash
npm run dev
# or
./dev.sh
```

This starts all services including the playground and other development tools.

**mprocs shortcuts:**
- `Tab` - Switch between processes
- `Space` - Start/stop a process
- `Ctrl+A` then `Q` - Quit all processes
- `Ctrl+A` then `K` - Kill a process
- `Arrow keys` - Navigate

### Option 3: Manual Database Management

If you need to manually manage databases:

```bash
# Start databases only
npm run db:up

# Stop databases
npm run db:down

# Restart databases
npm run db:restart

# View database logs
npm run db:logs
```

Note: `tana start` will automatically start required databases if they're not running.

## Environment Configuration

Create a `.env` file in the project root:

```bash
cp .env.example .env
```

Edit `.env` to configure:

```bash
# Database
DATABASE_URL='postgres://tana:[PASSWORD]@localhost:5432/tana'

# Redis
REDIS_URL='redis://localhost:6379'

# Service Ports
LEDGER_PORT=8080
CONTRACTS_PORT=8081
NODE_PORT=9933

# Environment
NODE_ENV=development
```

## Initialize Validator (Optional)

If you want to run as a validator node:

```bash
# Initialize validator with custom ports
bun cli/main.ts init validator --ws-port 9000 --http-port 9001

# Or use default ports
bun cli/main.ts init validator
```

This sets up your node to participate in consensus and validate blocks.

## Testing

```bash
# Run all tests
bun test

# Test specific service
bun run --filter @tana/ledger test

# Test Rust runtime
cd runtime && cargo test
```

## Common Development Tasks

### Database Management

```bash
# Start databases
npm run db:up

# Stop databases
npm run db:down

# View logs
npm run db:logs

# Reset databases (warning: deletes all data)
docker compose down -v && npm run db:up
```

### Building

```bash
# Build all services
npm run build

# Build Rust runtime
npm run build:runtime

# Build CLI binary
cd cli && bun run make
```

### Running Examples

```bash
# Test the runtime
bun run chaintest

# Run a contract
cd runtime
cargo run -- ../examples/default.ts
```

## Troubleshooting

### Port Already in Use

```bash
# Kill process on port 8080
lsof -ti:8080 | xargs kill -9

# Check what's using a port
lsof -i :8080
```

### Database Connection Errors

```bash
# Restart PostgreSQL
docker compose restart postgres

# Check logs
docker compose logs postgres

# Verify DATABASE_URL in .env
cat .env | grep DATABASE_URL
```

### Build Errors

```bash
# Clean and rebuild
rm -rf node_modules
bun install

# Rust
cd runtime
cargo clean
cargo build --release
```

## Project Structure

```
tana/
├── cli/             # CLI binary (TypeScript/Bun)
├── runtime/         # Contract execution (Rust)
├── ledger/          # Ledger service (TypeScript/Bun)
├── contracts/       # Contract executor (TypeScript/Bun)
├── node/            # Blockchain node (TypeScript/Bun)
├── websites/
│   ├── docs/        # Documentation (Astro/Starlight)
│   ├── playground/  # Contract playground (Astro/Svelte)
│   └── landing/     # Landing page (Next.js)
├── types/           # Shared TypeScript types
├── examples/        # Example contracts
└── docs/            # Technical documentation
```

## Next Steps

- Read [System Architecture](/contributing/architecture/) to understand how Tana works
- Explore [Core Concepts](/contributing/core/data-model/) for technical details
- Check out [Testing Guide](/contributing/testing/) for writing tests
