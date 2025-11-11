#!/bin/bash

# Setup script for GoQuant Upgrade System

set -e

echo "Setting up GoQuant Upgrade System..."

# Check Rust installation
if ! command -v rustc &> /dev/null; then
    echo "Rust not found. Please install Rust 1.75+"
    exit 1
fi

# Check Anchor installation
if ! command -v anchor &> /dev/null; then
    echo "Anchor not found. Installing Anchor..."
    cargo install --git https://github.com/coral-xyz/anchor avm --locked --force
    avm install latest
    avm use latest
fi

# Check Solana CLI
if ! command -v solana &> /dev/null; then
    echo "Solana CLI not found. Please install Solana CLI tools"
    exit 1
fi

# Check PostgreSQL
if ! command -v psql &> /dev/null; then
    echo "PostgreSQL not found. Please install PostgreSQL 12+"
    exit 1
fi

# Build Anchor program
echo "Building Anchor program..."
cd programs/upgrade-manager
anchor build
cd ../..

# Build backend
echo "Building backend service..."
cd backend
cargo build
cd ..

# Setup database
echo "Setting up database..."
createdb goquant_upgrades || true
psql goquant_upgrades < migrations/001_initial_schema.sql
psql goquant_upgrades < migrations/002_add_audit_log.sql

echo "Setup complete!"
echo ""
echo "Next steps:"
echo "1. Configure environment variables"
echo "2. Set up multisig wallet"
echo "3. Deploy program to devnet/testnet"
echo "4. Start backend service: cd backend && cargo run"

