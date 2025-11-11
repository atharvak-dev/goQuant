# GoQuant Program Upgrade & Migration System

A comprehensive system for managing safe, controlled upgrades of Solana programs with state migration capabilities for a decentralized perpetual futures exchange.

## Project Structure

```
goQuant/
├── programs/              # Anchor program (Solana smart contract)
│   └── upgrade-manager/
├── backend/              # Rust backend service
│   ├── src/
│   ├── migrations/
│   └── tests/
├── migrations/           # Database migrations
├── scripts/             # Utility scripts
├── tests/               # Integration tests
└── docs/                # Documentation
```

## Features

- **Upgrade Management**: Propose, approve, and execute program upgrades with multisig governance
- **Timelock Protection**: 48-hour timelock period for user safety
- **State Migration**: Safe migration of account data between program versions
- **Rollback Capability**: Emergency rollback mechanism for failed upgrades
- **Multisig Integration**: Integration with Squads Protocol for governance
- **Comprehensive APIs**: REST endpoints and WebSocket notifications
- **Audit Trail**: Complete history of all upgrade proposals and executions

## Quick Start

### Prerequisites

- Rust 1.75+
- Anchor 0.29+
- Solana CLI tools
- PostgreSQL 12+

### Setup

1. Install dependencies:
```bash
# Install Anchor
cargo install --git https://github.com/coral-xyz/anchor avm --locked --force
avm install latest
avm use latest

# Install Solana CLI
sh -c "$(curl -sSfL https://release.solana.com/stable/install)"
```

2. Build the Anchor program:
```bash
cd programs/upgrade-manager
anchor build
```

3. Set up the backend:
```bash
cd backend
cargo build
```

4. Set up the database:
```bash
# Create database
createdb goquant_upgrades

# Run migrations
cd migrations
psql goquant_upgrades < 001_initial_schema.sql
```

## Documentation

See the `docs/` directory for comprehensive documentation:
- Architecture overview
- Governance model
- Migration guide
- API documentation
- Operational runbook

## Testing

```bash
# Test Anchor program
cd programs/upgrade-manager
anchor test

# Test backend service
cd backend
cargo test
```

## License

Confidential - GoQuant Recruitment Assignment

# goQuant
# goQuant
