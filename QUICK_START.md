# GoQuant Project - Quick Start Guide

## Current Status

✅ **Installed & Ready:**
- Rust 1.91.0
- Anchor 0.32.1  
- Solana CLI 1.18.26
- Node.js dependencies

⚠️ **Needs Setup:**
- PostgreSQL database
- Database dependency conflict resolution
- Anchor program build configuration

## Quick Start Steps

### 1. Install PostgreSQL
Download and install from: https://www.postgresql.org/download/windows/

After installation, add PostgreSQL to your PATH or use full path to `psql.exe`.

### 2. Set Up Database
```powershell
# Create database
createdb goquant_upgrades

# Or using psql:
psql -U postgres -c "CREATE DATABASE goquant_upgrades;"

# Run migrations
psql goquant_upgrades < migrations\001_initial_schema.sql
psql goquant_upgrades < migrations\002_add_audit_log.sql
```

### 3. Resolve Dependency Conflict

The project has a dependency conflict between `sqlx` and `solana-client`. To resolve:

**Option A: Use Cargo Resolver (Recommended)**
Add to `backend/Cargo.toml`:
```toml
[package]
resolver = "2"
```

**Option B: Update Dependencies**
Update Solana dependencies to newer versions that are compatible with sqlx 0.8.

**Option C: Use Alternative Database Library**
Replace sqlx with `tokio-postgres` (requires rewriting database.rs).

### 4. Build Backend
```powershell
cd backend
$env:PATH += ";C:\Users\$env:USERNAME\.cargo\bin"
cargo build
```

### 5. Build Anchor Program
```powershell
# Add Solana tools to PATH
$env:PATH = ".\solana-release\bin;" + $env:PATH

# Use Anchor
$env:PATH += ";C:\Users\$env:USERNAME\.cargo\bin"
avm use latest

# Build
cd programs\upgrade-manager
anchor build
```

### 6. Start Backend Service
```powershell
cd backend
$env:DATABASE_URL = "postgresql://localhost/goquant_upgrades"
cargo run
```

The service will start on `http://0.0.0.0:3000`

## Environment Variables

Set these before running:
- `DATABASE_URL`: PostgreSQL connection string
- `SOLANA_RPC_URL`: Solana RPC endpoint (optional for local)
- `MULTISIG_ADDRESS`: Multisig wallet address (for production)

## Testing

```powershell
# Test Anchor program
cd programs\upgrade-manager
anchor test

# Test backend
cd backend
cargo test
```

## Troubleshooting

### Dependency Conflicts
If you see `serde_core` or `zeroize` conflicts:
1. Try `cargo update`
2. Add `resolver = "2"` to Cargo.toml
3. Consider updating Solana dependencies

### Anchor Build Issues
If `cargo build-sbf` not found:
- Ensure Solana tools are in PATH
- Or use full path: `.\solana-release\bin\cargo-build-sbf.exe`

### Database Connection Issues
- Verify PostgreSQL is running
- Check DATABASE_URL format: `postgresql://user:password@localhost/dbname`
- Ensure database exists and migrations are run

